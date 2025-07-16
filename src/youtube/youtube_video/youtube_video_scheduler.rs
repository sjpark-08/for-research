use std::time::Duration;
use chrono_tz::Tz;
use clokwerk::{AsyncScheduler, Job, TimeUnits};
use crate::app_state::AppState;

pub fn init_scheduler(app_state: AppState) {
    tokio::spawn(async move {
        let asia_seoul: Tz = "Asia/Seoul".parse().expect("Invalid timezone specified");
        let mut scheduler = AsyncScheduler::with_tz(asia_seoul);
        
        scheduler
            .every(1.day())
            .at("18:03")
            .run(move || {
                let app_state_clone = app_state.clone();
                async move {
                    match app_state_clone
                        .youtube_video_service
                        .run_video_collection_pipeline()
                        .await
                    {
                        Ok(_) => {
                            println!("[스케줄러] 데이터 수집 완료");
                        }
                        Err(e) => {
                            eprintln!("[스케줄러] 데이터 수집 실패 {:?}", e);
                        }
                    }
                }
            });
        
        loop {
            scheduler.run_pending().await;
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    });
}