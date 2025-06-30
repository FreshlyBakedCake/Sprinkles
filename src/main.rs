use zbus::{connection::Builder as ConnectionBuilder, interface as dbus_interface};

struct NotificationServer {}

#[dbus_interface(name = "org.freedesktop.Notifications")]
impl NotificationServer {
    async fn notify(
        &mut self,
        app_name: &str,
        replaces_id: u32,
        app_icon: &str,
        summary: &str,
        body: &str,
        actions: Vec<&str>,
        hints: std::collections::HashMap<&str, zbus::zvariant::Value<'_>>,
        expire_timeout: i32,
    ) -> u32 {
        println!(
            "Received notification (replacing: {replaces_id}) from {app_name}: {summary} - {body}",
        );
        println!("App Icon: {app_icon}");
        println!("Actions: {actions:?}");
        println!("Expires: {expire_timeout:?}");
        // println!("Hints: {hints:?}");
        return 1;
    }

    async fn close_notification(&mut self, id: u32) {}

    async fn get_capabilities(&self) -> Vec<String> {
        vec!["body".to_string(), "actions".to_string()]
    }

    async fn get_server_information(&self) -> (String, String, String, String) {
        (
            "Sprinkles".to_string(),
            "FreshlyBakedCake".to_string(),
            "1.0".to_string(),
            "1.2".to_string(),
        )
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let notification_server = NotificationServer {};

    let _conn = ConnectionBuilder::session()?
        .name("org.freedesktop.Notifications")?
        .serve_at("/org/freedesktop/Notifications", notification_server)?
        .build()
        .await?;

    std::future::pending::<()>().await;
    Ok(())
}
