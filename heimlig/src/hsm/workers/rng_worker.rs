use crate::common::jobs::Response::GetRandom;
use crate::common::jobs::{Error, Request, RequestId, Response};
use crate::common::limits::MAX_RANDOM_SIZE;
use crate::crypto::rng::{EntropySource, Rng};
use futures::{Sink, SinkExt, Stream, StreamExt};
use rand_core::RngCore;

pub struct RngWorker<
    'data,
    E: EntropySource,
    ReqSrc: Stream<Item = Request<'data>>,
    RespSink: Sink<Response<'data>>,
> {
    pub rng: Rng<E>,
    pub requests: ReqSrc,
    pub responses: RespSink,
}

impl<
        'data,
        E: EntropySource,
        ReqSrc: Stream<Item = Request<'data>> + Unpin,
        RespSink: Sink<Response<'data>> + Unpin,
    > RngWorker<'data, E, ReqSrc, RespSink>
{
    pub async fn execute(&mut self) -> Result<(), Error> {
        match self.requests.next().await {
            None => Ok(()), // Nothing to process
            Some(Request::GetRandom { request_id, output }) => {
                let response = self.get_random(request_id, output);
                self.responses
                    .send(response)
                    .await
                    .map_err(|_e| Error::Send)
            }
            _ => panic!("Encountered unexpected request"), // TODO: Integration error. Return error here instead?
        }
    }

    fn get_random<'a>(&mut self, request_id: RequestId, output: &'a mut [u8]) -> Response<'a> {
        if output.len() >= MAX_RANDOM_SIZE {
            return Response::Error {
                request_id,
                error: Error::RequestTooLarge,
            };
        }
        self.rng.fill_bytes(output);
        GetRandom {
            request_id,
            data: output,
        }
    }
}
