use crate::world::components::Job;

pub struct JobInfo {
    pub job: Job,
    pub daily_income: f32,
    pub danger: f32,
    pub energy_cost: f32,
}

pub fn job_info(job: Job) -> JobInfo {
    match job {
        Job::Guard => JobInfo {
            job,
            daily_income: 0.8,
            danger: 0.1,
            energy_cost: 0.3,
        },
        Job::Trader => JobInfo {
            job,
            daily_income: 1.0,
            danger: 0.05,
            energy_cost: 0.2,
        },
        Job::ArtifactHunter => JobInfo {
            job,
            daily_income: 1.5,
            danger: 0.4,
            energy_cost: 0.5,
        },
        Job::Bandit => JobInfo {
            job,
            daily_income: 1.8,
            danger: 0.6,
            energy_cost: 0.4,
        },
        Job::Unemployed => JobInfo {
            job,
            daily_income: 0.0,
            danger: 0.0,
            energy_cost: 0.0,
        },
        Job::Hunter => JobInfo {
            job,
            daily_income: 1.2,
            danger: 0.35,
            energy_cost: 0.45,
        },
        Job::Scavenger => JobInfo {
            job,
            daily_income: 0.9,
            danger: 0.3,
            energy_cost: 0.35,
        },
        Job::Courier => JobInfo {
            job,
            daily_income: 0.7,
            danger: 0.2,
            energy_cost: 0.4,
        },
        Job::Resident => JobInfo {
            job,
            daily_income: 0.3,
            danger: 0.02,
            energy_cost: 0.15,
        },
    }
}
