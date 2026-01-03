pub enum PowerAction {
    Reboot,
    Shutdown,
}

pub fn power(action: PowerAction) {
    let cmd = match action {
        PowerAction::Reboot => "reboot",
        PowerAction::Shutdown => "poweroff",
    };

    let _ = std::process::Command::new("systemctl").arg(cmd).spawn();
}
