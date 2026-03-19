use std::process::{Command, Stdio};
// use tokio::process::Command; // 高并发服务必须用 async; 高并发服务必须用 async
use std::time::{Duration, Instant};
use std::io::Read;
use std::thread;

#[derive(Debug)]
pub struct CommandResult {
    pub stdout: String,
    pub stderr: String,
    pub status: i32,
}

#[derive(Debug)]
pub enum CommandError {
    IoError(std::io::Error),
    Timeout,
    Utf8Error(std::string::FromUtf8Error),
}

impl From<std::io::Error> for CommandError {
    fn from(e: std::io::Error) -> Self {
        CommandError::IoError(e)
    }
}

impl From<std::string::FromUtf8Error> for CommandError {
    fn from(e: std::string::FromUtf8Error) -> Self {
        CommandError::Utf8Error(e)
    }
}

pub struct CommandExecutor;

impl CommandExecutor {
    /// 通用执行
    pub fn exec(
        program: &str,
        args: &[&str],
        timeout: Option<Duration>,
    ) -> Result<CommandResult, CommandError> {
        // 1️⃣ 支持环境变量
        // Command::new(program).env("KEY", "VALUE");
        // 2️⃣ 指定工作目录
        // Command::new(program).current_dir("/tmp")
        // 3️⃣ 防命令注入（非常重要）❌ 危险: exec_shell(&format!("rm -rf {}", user_input));
        // Command::new("rm").arg("-rf").arg(user_input)
        let mut child = Command::new(program)
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let start = Instant::now();

        loop {
            if let Some(status) = child.try_wait()? {
                let mut stdout = Vec::new();
                let mut stderr = Vec::new();

                if let Some(mut out) = child.stdout.take() {
                    out.read_to_end(&mut stdout)?;
                }

                if let Some(mut err) = child.stderr.take() {
                    err.read_to_end(&mut stderr)?;
                }

                return Ok(CommandResult {
                    stdout: String::from_utf8(stdout)?,
                    stderr: String::from_utf8(stderr)?,
                    status: status.code().unwrap_or(-1),
                });
            }

            if let Some(t) = timeout {
                if start.elapsed() > t {
                    child.kill()?;
                    return Err(CommandError::Timeout);
                }
            }

            thread::sleep(Duration::from_millis(50));
        }
    }

    /// Windows CMD
    pub fn exec_cmd(cmd: &str) -> Result<CommandResult, CommandError> {
        #[cfg(target_os = "windows")]
        {
            Self::exec("cmd", &["/C", cmd], None)
        }

        #[cfg(not(target_os = "windows"))]
        {
            Err(CommandError::IoError(std::io::Error::new(
                std::io::ErrorKind::Other,
                "cmd only supported on Windows",
            )))
        }
    }

    /// PowerShell
    pub fn exec_powershell(cmd: &str) -> Result<CommandResult, CommandError> {
        #[cfg(target_os = "windows")]
        {
            Self::exec("powershell", &["-Command", cmd], None)
        }

        #[cfg(not(target_os = "windows"))]
        {
            Err(CommandError::IoError(std::io::Error::new(
                std::io::ErrorKind::Other,
                "PowerShell only supported on Windows",
            )))
        }
    }

    /// Linux/macOS shell
    pub fn exec_shell(cmd: &str) -> Result<CommandResult, CommandError> {
        #[cfg(target_family = "unix")]
        {
            Self::exec("sh", &["-c", cmd], None)
        }

        #[cfg(not(target_family = "unix"))]
        {
            Err(CommandError::IoError(std::io::Error::new(
                std::io::ErrorKind::Other,
                "shell only supported on Unix",
            )))
        }
    }

    /// 自动选择
    pub fn exec_auto(cmd: &str) -> Result<CommandResult, CommandError> {
        #[cfg(target_os = "windows")]
        {
            Self::exec_cmd(cmd)
        }

        #[cfg(target_family = "unix")]
        {
            Self::exec_shell(cmd)
        }
    }
}