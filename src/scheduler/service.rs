use std::collections::{HashMap};
use std::sync::{Arc, Mutex};
// use

use log::debug;
use log::info;
use tonic::{ Request, Response, Status};

use crate::scheduler::proto::CheckExecutorsRequest;
use crate::scheduler::proto::CheckExecutorsResponse;
use crate::executor::proto::gin_executor_service_client::GinExecutorServiceClient;
use crate::executor::proto::Empty;
use crate::scheduler::proto::gin_scheduler_service_server::GinSchedulerService;

use crate::scheduler::proto::{RegisterExecutorResponse,UnregisterExecutorResponse,RegisterExecutorRequest, SubmitJobRequest, SubmitJobResponse, UnregisterExecutorRequest};

use std::sync::atomic::{AtomicUsize, Ordering};

use crate::common::common::stage::StageType;

pub struct IdGenerator {
    next_id: AtomicUsize,
}

impl IdGenerator {
    pub fn new() -> Self {
        Self {
            next_id: AtomicUsize::new(0),
        }
    }

    pub fn generate_id(&self) -> usize {
        self.next_id.fetch_add(1, Ordering::SeqCst)
    }
}

// Define a struct to hold the state of the scheduler
// impl Scheduler {
//     // Delegate a job to the next available executor
//     async fn _delegate_job(&mut self) -> Result<(), Status> {
//         // Acquire the job queue lock
//         let lock = self.job_queue_lock.clone();
//         let _guard = lock.lock().unwrap();

//         // Check if there are any pending jobs
//         let job = match self.pending_jobs.pop_front() {
//             Some(job) => job,
//             None => return Ok(()),
//         };

//         // Find the next available executor
//         let mut selected_executor = None;
//         for (uri, connected) in &self.executors {
//             if *connected.borrow() {
//                 selected_executor = Some(uri);
//                 break;
//             }
//         }

//         // If no executor is available, put the job back on the pending jobs queue and return
//         let executor_uri = match selected_executor {
//             Some(uri) => uri,
//             None => {
//                 self.pending_jobs.push_back(job);
//                 return Ok(());
//             }
//         };

//         // Send the job to the selected executor
//         let client = match GinExecutorServiceClient::connect(executor_uri.clone()).await {
//             Ok(client) => client,
//             Err(_) => {
//                 // The executor is no longer connected
//                 self.executors
//                     .insert(executor_uri.clone(), RefCell::new(false));
//                 return Ok(());
//             }
//         };

//         debug!(
//             "Job {} (not really) delegated to executor {}",
//             job.id, executor_uri
//         );
//         // let request = tonic::Request::new(job.clone());
//         // match client.(request).await {
//         //     Ok(_) => {
//         //         // Job was successfully delegated
//         //     }
//         //     Err(_) => {
//         //         // The executor is no longer connected
//         //         self.executors.insert(executor_uri.clone(), false);
//         //         return Ok(());
//         //     }
//         // };

//         Ok(())
//     }

// }

// Implement the SchedulerService gRPC service
// #[derive(Debug)]

pub struct Scheduler {
    executors: Arc<Mutex<HashMap<String, bool>>>,

    // id_generator: Arc<IdGenerator>,
}

impl Scheduler {
    pub fn new() -> Self {
        Scheduler {
            executors: Arc::new(Mutex::new(HashMap::new())),
            // id_generator: Arc::new(IdGenerator::new()),
        }
    }
}
// Implement the service methods
#[tonic::async_trait]
impl GinSchedulerService for Scheduler {
    async fn register_executor(
        &self,
        _request: Request<RegisterExecutorRequest>,
    ) -> Result<Response<RegisterExecutorResponse>, Status> {
        let uri = _request.get_ref().clone();

        // Add the executor to the list of connected executors
        let mut executors = self.executors.lock().unwrap();
        executors.insert(uri.executor_uri.to_owned(), true);

        let response = RegisterExecutorResponse { success: true };
        info!(
            "Executor {} connected!",
            _request.get_ref().executor_uri.clone()
        );
        Ok(Response::new(response))
    }

    async fn unregister_executor(
        &self,
        _request: Request<UnregisterExecutorRequest>,
    ) -> Result<Response<UnregisterExecutorResponse>, Status> {
        todo!()
    }

    async fn submit_job(
        &self,
        _request: Request<SubmitJobRequest>,
    ) -> Result<Response<SubmitJobResponse>, Status> {
        let graph =  &_request.get_ref().plan;
        
        for stage in graph.iter() {
            match &stage.stage_type {
                Some(StageType::Action(method_type)) => {
                    debug!("action {}", method_type);
                },
                Some(StageType::Filter(_method_type)) => {
                    debug!("filter");
                },
                Some(StageType::Select(_method_type)) => {
                    debug!("select");
                },
                None => debug!("No valid method"),
            }
        }
        todo!();
        // todo!()
    }

    async fn check_executors(
        &self,
        _request: Request<CheckExecutorsRequest>,
    ) -> Result<Response<CheckExecutorsResponse>, Status> {
        debug!("Checking executors!");
    
        let mut executor_stats = HashMap::<String, bool>::new();
        let executors_copy: HashMap<String,bool> = {
            let executors_guard = self.executors.lock().unwrap();
            (*executors_guard).clone()
        };
        for (uri, executor) in executors_copy.iter() {
            debug!("{} {}", uri.clone(), executor.clone());
            let mut client = match GinExecutorServiceClient::connect(uri.clone()).await{
                Ok(client) => client,
                Err(_) => {
                    // Executor is not reachable
                    continue;
                }
            };
            match client.heartbeat(Empty {}).await {
                Ok(_) => {
                    // Executor is still connected
                    *executor_stats.entry(uri.clone()).or_insert(true) = true;
                }
                Err(_) => {
                    // Executor is not reachable
                    *executor_stats.entry(uri.clone()).or_insert(false) = false;
                }
            };
        }
        {
            let mut executors_guard = self.executors.lock().unwrap();
            (*executors_guard) = executor_stats.clone();
        }
        let response = CheckExecutorsResponse {
            executor_status: executor_stats,
        };
        Ok(Response::new(response))
    }
    
    
}