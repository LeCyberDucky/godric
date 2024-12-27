use color_eyre::Result;
use thirtyfour as tf;

#[derive(
    Copy, Clone, Debug, strum_macros::Display, strum_macros::EnumString, strum_macros::EnumIter,
)]
pub enum Browser {
    Chrome,
    Chromium,
    Edge,
    Firefox,
    #[strum(to_string = "Internet Explorer")]
    InternetExplorer,
    Opera,
    Safari,
}

impl Browser {
    pub fn driver(&self) -> String {
        let mut name = match self {
            Browser::Chrome => todo!(),
            Browser::Chromium => todo!(),
            Browser::Edge => todo!(),
            Browser::Firefox => "geckodriver".to_string(),
            Browser::InternetExplorer => todo!(),
            Browser::Opera => todo!(),
            Browser::Safari => todo!(),
        };

        if cfg!(target_os = "windows") {
            name += ".exe";
        }

        name
    }
}

#[derive(Clone, Debug)]
pub struct DriverConfig {
    pub browser: Browser,
    pub driver_address: std::net::SocketAddrV4,
    pub headless: bool,
}

pub struct Connection {
    pub browser: tf::WebDriver,
    driver: Option<std::process::Child>,
}

impl std::fmt::Debug for Connection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // tf::WebDriver doesn't like being formatted as Debug
        f.debug_struct("Connection")
            .field("driver", &self.driver)
            .finish()
    }
}

impl Connection {
    pub async fn new(config: &DriverConfig) -> Result<Self> {
        // Don't attempt to launch the driver, if a corresponding process already exists
        let driver = (sysinfo::System::processes_by_exact_name(
            &sysinfo::System::new_all(),
            std::ffi::OsStr::new(&config.browser.driver()),
        )
        .count()
            < 1)
        .then_some(Self::launch_driver(
            &config.browser,
            &config.driver_address,
        )?);

        let browser = Self::launch_browser(config).await?;

        Ok(Self { browser, driver })
    }

    fn launch_driver(
        browser: &Browser,
        address: &std::net::SocketAddrV4,
    ) -> Result<std::process::Child> {
        Ok(std::process::Command::new(browser.driver())
            .args([
                "--host",
                &address.ip().to_string(),
                "--port",
                &address.port().to_string(),
            ])
            .stdout(std::process::Stdio::null())
            .spawn()?)
    }

    async fn launch_browser(config: &DriverConfig) -> Result<tf::WebDriver> {
        let driver_address =
            url::Url::parse(&("http://".to_string() + config.driver_address.to_string().as_str()))?;

        let mut browser_capabilities = match config.browser {
            Browser::Chrome => todo!(),
            Browser::Chromium => todo!(),
            Browser::Edge => todo!(),
            Browser::Firefox => tf::DesiredCapabilities::firefox(),
            Browser::InternetExplorer => todo!(),
            Browser::Opera => todo!(),
            Browser::Safari => todo!(),
        };
        if config.headless {
            browser_capabilities.set_headless()?
        }

        Ok(tf::WebDriver::new(driver_address.as_str(), browser_capabilities).await?)
    }
}
