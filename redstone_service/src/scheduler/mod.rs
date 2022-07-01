use chrono::Utc;
use cron::Schedule;
use std::{borrow::BorrowMut, str::FromStr};

use tokio::{sync::mpsc::UnboundedReceiver, task::JoinHandle};

pub async fn run_upload_jobs(
    new_job_receiver: &mut UnboundedReceiver<UpdateJob>,
) -> Result<(), std::boxed::Box<dyn std::fmt::Debug>> {
    let _ = tokio::join!(
        run_stored_jobs(get_jobs()),
        listen_for_new_jobs(new_job_receiver.borrow_mut())
    );
    Ok(())
}

#[derive(Debug)]
pub struct UpdateJob {
    pub backup_id: usize,
    pub cron_expr: String,
}

fn get_jobs() -> Vec<UpdateJob> {
    vec![
        UpdateJob {
            cron_expr: String::from("*/3 * * * * * *"),
            backup_id: 1,
        },
        UpdateJob {
            cron_expr: String::from("*/6 * * * * * *"),
            backup_id: 2,
        },
    ]
}

async fn run_stored_jobs(jobs: Vec<UpdateJob>) -> Vec<JoinHandle<Result<(), cron::error::Error>>> {
    let mut job_vec = Vec::new();
    for job in jobs {
        let handle = run_job(job);
        job_vec.push(handle);
    }
    job_vec
}

fn run_job(job: UpdateJob) -> JoinHandle<Result<(), cron::error::Error>> {
    tokio::task::spawn(async move {
        let schedule = Schedule::from_str(job.cron_expr.as_str())?;
        for date_time in schedule.after(&Utc::now()) {
            sleep_until_exec_time(date_time).await;
        }
        Ok(())
    })
}

async fn listen_for_new_jobs(
    new_job_receiver: &mut UnboundedReceiver<UpdateJob>,
) -> Result<(), cron::error::Error> {
    while let Some(job) = new_job_receiver.recv().await {
        // TODO: store_job(job);
        run_job(job);
    }
    Ok(())
}

async fn sleep_until_exec_time(date_time: chrono::DateTime<Utc>) -> () {
    let duration = tokio::time::Duration::from_secs((date_time - Utc::now()).num_seconds() as u64);
    tokio::time::sleep(duration).await;
}
