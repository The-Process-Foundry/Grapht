//! Use an external copy of redis graph as a backend

use super::Backend;
use crate::local::*;

use redis::{FromRedisValue, RedisError, RedisResult, Value};
use rust_decimal::Decimal;

#[derive(Debug, Clone)]
pub struct RedisGraph {
  /// The configuration for interacting with Redis
  config: RedisGraphConfig,

  /// A RedisGraph Client
  client: ClientState,
}

impl RedisGraph {
  pub fn new(config: RedisGraphConfig) -> RedisGraph {
    RedisGraph {
      config,
      client: ClientState::Closed,
    }
  }

  pub fn open(&mut self) -> GraphtResult<()> {
    match self.client {
      ClientState::Open(_) => info!("Tried to open an already open Redis Graph connection"),

      ClientState::Error(_) | ClientState::Closed => {
        let url = self.config.get_url();
        info!("Trying to open a client for RedisGraph at {:?}", url);
        match redis::Client::open(url) {
          Ok(client) => self.client = ClientState::Open(client),
          Err(err) => self.client = ClientState::Error(err.to_string()),
        }
      }
    }
    Ok(())
  }

  pub fn close(&mut self) -> GraphtResult<()> {
    match self.client {
      ClientState::Open(_) | ClientState::Error(_) => self.client = ClientState::Closed,
      ClientState::Closed => (),
    }
    Ok(())
  }

  fn get_connection(&mut self) -> GraphtResult<redis::Connection> {
    let client = match &self.client {
      ClientState::Open(client) => client,
      _ => {
        self.open()?;
        if let ClientState::Open(client) = &self.client {
          client
        } else {
          unreachable!("There should always be an open connection at this point")
        }
      }
    };
    Ok(client.get_connection()?)
  }

  /// Send a message
  pub fn send_(&mut self, msg: &str) -> GraphtResult<QuerySet> {
    // Parses and validates the message
    let query = GQuery::parse(msg)?;

    let mut cmd = redis::cmd("GRAPH.QUERY");
    cmd.arg(self.config.get_name()).arg(msg);
    info!(
      "Sending command to {}:{:?}\n\t{:?}",
      self.name(),
      self.config.get_name(),
      msg
    );

    let mut conn = self.get_connection()?;

    // info!("Finished Build. Processing result");
    // Let the redis module parse the result for now. Premature to write a custom parser
    let raw: RawValue = cmd.query(&mut conn)?;
    raw.parse(&query)
  }
}

// pub fn recv(&mut self, )

impl Backend for RedisGraph {
  type RawResponse = RawValue;

  fn name(&self) -> String {
    self.config.get_name()
  }

  fn send(&mut self, msg: &str) -> GraphtResult<Self::RawResponse> {
    todo!("Send for RedisGraph")
  }

  fn parse(&mut self, query_set: &mut QuerySet, response: RawValue) -> GraphtResult<()> {
    todo!("RedisGraph::parse")
  }
}

#[derive(Clone)]
pub enum ClientState {
  Closed,
  Open(redis::Client),
  Error(String),
}

impl fmt::Debug for ClientState {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Closed => write!(f, "Closed"),
      Self::Open(_) => write!(f, "Open"),
      Self::Error(arg0) => f.debug_tuple("Error").field(arg0).finish(),
    }
  }
}

#[derive(Debug, Clone)]
pub struct RedisGraphConfig {
  name: String,

  /// A redis address
  url: String,
}

impl RedisGraphConfig {
  pub fn new(name: &str, url: &str) -> RedisGraphConfig {
    RedisGraphConfig {
      name: name.to_string(),
      url: url.to_string(),
    }
  }
  pub fn get_name(&self) -> String {
    self.name.clone()
  }

  pub fn get_url(&self) -> String {
    self.url.clone()
  }
}

impl ValueParser for RawValue {
  fn parse(&self, query: &GQuery) -> GraphtResult<QuerySet> {
    use result_ast::*;

    fn handle_stat(value: &Value) -> GraphtResult<RedisStatistic> {
      // info!("Parsing statistic: {:?}", value);
      match value {
        Value::Data(inner) => Ok(RedisStatistic::parse(inner)?),
        _ => Err(err!(
          TypeMismatch,
          "Invalid type: Expected a statistic string here but received: {:?}",
          value
        ))?,
      }
    }

    fn upsert_statement(query_set: &mut QuerySet, statement: Response) -> GraphtResult<()> {
      use query_set::Statistic as Stat;
      for stat in statement.stats {
        match stat {
          RedisStatistic::NodesCreated(count) => query_set.add_stat(Stat::NodesCreated(count))?,
          RedisStatistic::NodesDeleted(count) => query_set.add_stat(Stat::NodesDeleted(count))?,
          RedisStatistic::RelationshipsCreated(count) => {
            query_set.add_stat(Stat::EdgesCreated(count))?
          }
          RedisStatistic::PropertiesAdded(count) => {
            query_set.add_stat(Stat::PropertiesSet(count))?
          }
          RedisStatistic::LabelsAdded(count) => query_set.add_stat(Stat::LabelsAdded(count))?,
          RedisStatistic::CachedExecution(_) => (),
          RedisStatistic::ExecutionTime(time) => query_set.add_stat(Stat::ExecutionTime(
            Decimal::from_f32_retain(time).unwrap().round_dp(6),
          ))?,
          _ => todo!("Other Stats: {:?}", stat),
        }
      }
      Ok(())
    }

    fn handle_statement(value: &Value, query_set: &mut QuerySet) -> GraphtResult<()> {
      let mut statement = Response::new();
      match value {
        Value::Bulk(stats) => {
          info!("Split bulk into stats: {:?}", stats);
          for stat in stats {
            statement.add_stat(handle_stat(stat)?)?;
            // stat.upsert(query_set)?;
          }
        }

        // Generic u8 data value
        Value::Data(value) => {
          let string = std::str::from_utf8(value)?;
          // info!("Processing string value: '{:?}'", string);
        }
        _ => {
          unimplemented!("QuerySet::FromRedisValue::from_redis_value - Match not implemented yet")
        }
      }
      // info!("Finished processing statement response:\n\t{:?}", statement);

      upsert_statement(query_set, statement)
    }

    info!("Parsing the result from query: {:?}", query);
    let mut query_set = QuerySet::new(query.clone());

    match &self.0 {
      Value::Bulk(responses) => {
        info!("response length: {:?}", responses.len());
        for response in responses {
          handle_statement(&response, &mut query_set)?;
        }
      }

      _ => todo!(
        "Expecting a list of Redis Graph results values. Instead got:\n\t{:?}",
        self
      ),
    }

    Ok(query_set)
  }
}
/*
  fn upsert(&self, query: GQuery, query_set: &mut QuerySet) -> GraphtResult<()> {



  }
}
 */

/// NewType wrapper so we can just return the unparsed value
///
/// FromRedisValue cannot handle using the query to determine the data type, so we package it up
/// and send it someplace that can
#[derive(Debug, Clone)]
pub struct RawValue(Value);

impl FromRedisValue for RawValue {
  fn from_redis_value(v: &Value) -> RedisResult<RawValue> {
    Ok(RawValue(v.clone()))
  }

  fn from_redis_values(v: &[Value]) -> RedisResult<Vec<RawValue>> {
    Ok(v.iter().map(|val| RawValue(val.clone())).collect())
  }
}

pub mod result_ast {
  use crate::errors::{GQueryError, Result as GraphtResult};
  use nom::{character::complete::alpha1, IResult};

  pub trait Nomical: core::fmt::Debug + Clone + Sized {
    type Error: core::fmt::Debug;

    fn parse(input: &Vec<u8>) -> Result<Self, Self::Error>;

    /// Print the given object out as a string
    ///
    /// TODO: This needs designing, as there needs to be formatting options
    fn to_string(&self) -> String {
      todo!("Nomical::to_string needs to be designed")
    }
  }

  // /// Pointer to a substring in the raw result
  // #[derive(Debug, Clone)]
  // pub struct Span {
  //   pub(crate) lo: u32,
  //   pub(crate) high: u32,
  // }

  /// A deserialized result from a command/query sent to RedisGraph
  pub struct RedisResult {
    raw: Vec<u8>,

    /// Parsed values
    parsed: Vec<GraphtResult<Response>>,
  }

  /// An enumeration of all the token types, both terminal and non-terminal.
  pub enum Tokens {
    /// An empty item, explicitly being used as a place holder
    ///
    /// For example, a string "A full statement ended by a semicolon;;" is two statements, with the
    /// latter being Null
    Null,

    /// Any form of whitespace (spaces, newlines, tabs,)
    WhiteSpace,

    /// A response to a statement
    StatementResponse(Response),

    ///
    Statistic,
  }

  /// A response to a single statement sent to the Redis server
  #[derive(Debug, Clone)]
  pub struct Response {
    // data: HashMap<String, Vec<Column>>,
    /// A list of statistics returned from RedisGraph. These will be rolled up for the QuerySet
    pub stats: Vec<RedisStatistic>,
  }

  impl Response {
    pub fn new() -> Response {
      Response {
        // data: HashMap::new(),
        stats: Vec::new(),
      }
    }

    /// Add a new statistic about the result
    pub fn add_stat(&mut self, stat: RedisStatistic) -> GraphtResult<()> {
      self.stats.push(stat);
      Ok(())
    }
  }

  impl Nomical for Response {
    type Error = GQueryError;

    fn parse(value: &Vec<u8>) -> Result<Self, Self::Error> {
      todo!()
    }
  }

  pub struct DataRow {}

  /// Parse each possible stat that can be returned as a string from RedisGraph
  #[derive(Debug, Clone)]
  pub enum RedisStatistic {
    /// A count of how many new nodes were added to the graph
    NodesCreated(i32),

    /// A count of how many nodes were removed from the graph
    NodesDeleted(i32),

    /// A count of how many new edges were added to the graph
    RelationshipsCreated(i32),

    /// A count of how many unique properties were added to Node and Relationship entities
    PropertiesAdded(i32),

    /// New labels to tag various nodes
    LabelsAdded(i32),

    /// Whether the query run has been cached
    /// THINK: Is the query compilation cached, or the actual result? What about partials,
    ///   multiple clauses where some are cached and some not.
    CachedExecution(i32),

    /// Time it took to finish the query in milliseconds
    ExecutionTime(f32),

    /// Didn't match any known statistic
    Unknown(String),
  }

  impl Nomical for RedisStatistic {
    type Error = GQueryError;

    fn parse(value: &Vec<u8>) -> Result<Self, Self::Error> {
      use crate::local::*;
      use nom::{
        // branch::alt,
        bytes::complete::{take, take_till, take_until},
        character::complete::multispace0,
        error::Error as NomError,
        number::complete::float,
        sequence::tuple,
      };

      //---- Branches
      fn paren_match(left: &[u8], descriptor: &[u8]) -> Result<RedisStatistic, GQueryError> {
        let matched: IResult<&[u8], (&[u8], &[u8]), NomError<&[u8]>> =
          tuple((take_until(")"), take(1 as u16)))(left);
        match matched {
          Ok((_left, (value, _))) if value[0] as char == '*' => {
            Ok(RedisStatistic::NodesCreated(-1))
          }
          Ok((_left, (value, _))) => {
            let inner = str::from_utf8(value)?;
            // info!("Inner is {:?}", inner);
            Ok(RedisStatistic::NodesCreated(inner.parse().unwrap()))
          }
          Err(err) => {
            let msg = format!(
              "Could not find a right parenthesis to match {:?}\n{:?}",
              left, err
            );
            error!("{}", msg);
            Err(err!(ParsingError, "{}", msg))?
          }
        }
      }

      fn colon_match(left: &[u8], descriptor: &str) -> Result<RedisStatistic, GQueryError> {
        // info!("Matched Paren Desc: {:?}, left {:?}", descriptor, left);
        let count: IResult<&[u8], (&[u8], f32), NomError<&[u8]>> =
          tuple((multispace0, float))(left);
        if let Err(err) = count {
          return Err(err!(
            ParsingError,
            "Couldn't get a value for descriptor {:?} from {:?} because {:?}",
            descriptor,
            left,
            err
          ));
        };
        let count = count.unwrap();

        match descriptor {
          "Nodes created" => Ok(RedisStatistic::NodesCreated(count.1 .1 as i32)),
          "Nodes deleted" => Ok(RedisStatistic::NodesDeleted(count.1 .1 as i32)),
          "Relationships created" => Ok(RedisStatistic::RelationshipsCreated(count.1 .1 as i32)),
          "Query internal execution time" => Ok(RedisStatistic::ExecutionTime(count.1 .1 as f32)),
          "Cached execution" => Ok(RedisStatistic::CachedExecution(count.1 .1 as i32)),
          "Labels added" => Ok(RedisStatistic::LabelsAdded(count.1 .1 as i32)),
          "Properties set" => Ok(RedisStatistic::PropertiesAdded(count.1 .1 as i32)),
          unmatched => {
            warn!("Unmatched statistic string: {:?}", unmatched);
            Err(err!(
              NotFound,
              "Received an unknown type of statistic: {:?}",
              unmatched
            ))
          }
        }
      }
      // let colon_count = tuple((take_until(":"), nom_char(':'), multispace0, float))(value.as_slice());
      // let paren_count = tuple(take_until("("), )
      // let parsed

      let matched: IResult<&[u8], (&[u8], &[u8]), NomError<&[u8]>> = tuple((
        take_till(|c: u8| [':', '('].contains(&(c as char))),
        take(1 as u16),
      ))(value.as_slice());
      match matched {
        Ok((left, (descriptor, sep))) if sep[0] as char == ':' => {
          colon_match(left, str::from_utf8(descriptor)?)
        }
        Ok((left, (descriptor, sep))) if sep[0] as char == '(' => paren_match(left, descriptor),
        Err(err) => {
          let msg = format!("Could not parse the statistic from {:?}\n{:?}", value, err);
          error!("{}", msg);
          Err(err!(ParsingError, "{}", msg))?
        }
        unmatched => {
          warn!("Unmatched statistic string: {:?}", unmatched);
          Err(err!(
            NotFound,
            "Received an unknown type of statistic: {:?}",
            unmatched
          ))
        }
      }
    }
  }
}
