use notify_rust::Notification;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    println!("喝水提醒服务已启动，每 40 分钟提醒一次...");

    loop {
        // 1. 等待 40 分钟
        // 40 分钟 * 60 秒
        sleep(Duration::from_secs(40 * 60)).await;

        // 2. 发送通知
        send_notification();
    }
}

fn send_notification() {
    Notification::new()
        .summary("该喝水啦宝宝")
        .body("已经过去 40 分钟了，喝口水活动一下吧。")
        .icon("dialog-information")
        .show()
        .unwrap();
}