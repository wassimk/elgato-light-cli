use elgato_keylight::KeyLight;
use std::error::Error;
use std::net::Ipv4Addr;
use std::str::FromStr;
use structopt::StructOpt;

const DEFAULT_IP_ADDRESS: &str = "192.168.0.25";

#[derive(StructOpt, Debug)]
#[structopt(
    name = "elgato light",
    about = "A command line interface for controlling an Elgato light by its IP address"
)]
enum ElgatoLight {
    #[structopt(about = "Turns the light on with specified brightness and temperature")]
    On {
        #[structopt(
            short = "b",
            long = "brightness",
            default_value = "10",
            help = "Set the brightness level (0-100)"
        )]
        brightness: u8,

        #[structopt(
            short = "t",
            long = "temperature",
            default_value = "3000",
            help = "Set the color temperature (2900-7000)"
        )]
        temperature: u32,

        #[structopt(short = "i", long = "ip-address", default_value = DEFAULT_IP_ADDRESS, help = "Specify the IP address of the Elgato Light")]
        ip_address: String,
    },
    #[structopt(about = "Turns the light off")]
    Off {
        #[structopt(short = "i", long = "ip-address", default_value = DEFAULT_IP_ADDRESS, help = "Specify the IP address of the Elgato Light")]
        ip_address: String,
    },
    #[structopt(
        about = "Changes the brightness of the light. Use -100 to 100. Use -- to pass negative arguments."
    )]
    Brightness {
        #[structopt(help = "Change the brightness level (-100 to 100)")]
        brightness: i8,

        #[structopt(short = "i", long = "ip-address", default_value = DEFAULT_IP_ADDRESS, help = "Specify the IP address of the Elgato Light")]
        ip_address: String,
    },
    #[structopt(about = "Sets the temperature of the light")]
    Temperature {
        #[structopt(help = "Set the color temperature (2900-7000)")]
        temperature: u32,

        #[structopt(short = "i", long = "ip-address", default_value = DEFAULT_IP_ADDRESS, help = "Specify the IP address of the Elgato Light")]
        ip_address: String,
    },
    #[structopt(about = "Gets the status of the light")]
    Status {
        #[structopt(short = "i", long = "ip-address", default_value = DEFAULT_IP_ADDRESS, help = "Specify the IP address of the Elgato Light")]
        ip_address: String,
    },
}

impl ElgatoLight {
    fn ip_address(&self) -> Result<Ipv4Addr, Box<dyn Error>> {
        let ip_str = match self {
            ElgatoLight::On { ip_address, .. }
            | ElgatoLight::Off { ip_address }
            | ElgatoLight::Brightness { ip_address, .. }
            | ElgatoLight::Temperature { ip_address, .. }
            | ElgatoLight::Status { ip_address } => ip_address,
        };

        Ipv4Addr::from_str(ip_str).map_err(|_| "Invalid IP address format".into())
    }

    async fn get_keylight(ip_address: Ipv4Addr) -> Result<KeyLight, Box<dyn Error>> {
        let keylight = KeyLight::new_from_ip("Elgato Light", ip_address, None).await?;
        Ok(keylight)
    }

    async fn ensure_light_on(keylight: &mut KeyLight) -> Result<(), Box<dyn Error>> {
        let status = keylight.get().await?;
        if status.lights[0].on == 0 {
            keylight.set_power(true).await?;
        }
        Ok(())
    }

    async fn run(&self, mut keylight: KeyLight) -> Result<(), Box<dyn Error>> {
        match self {
            ElgatoLight::On {
                brightness,
                temperature,
                ..
            } => {
                keylight.set_power(true).await?;
                keylight.set_brightness(*brightness).await?;
                keylight.set_temperature(*temperature).await?;
            }
            ElgatoLight::Off { .. } => {
                keylight.set_power(false).await?;
            }
            ElgatoLight::Brightness { brightness, .. } => {
                ElgatoLight::ensure_light_on(&mut keylight).await?;
                let status = keylight.get().await?;
                let current_brightness = status.lights[0].brightness;
                let new_brightness = ((current_brightness as i8) + *brightness).clamp(0, 100) as u8;
                keylight.set_brightness(new_brightness).await?;
            }
            ElgatoLight::Temperature { temperature, .. } => {
                ElgatoLight::ensure_light_on(&mut keylight).await?;
                keylight.set_temperature(*temperature).await?;
            }
            ElgatoLight::Status { .. } => {
                let status = keylight.get().await?;
                println!("{:?}", status);
            }
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = ElgatoLight::from_args();
    let ip_address = args.ip_address()?;
    let keylight = ElgatoLight::get_keylight(ip_address).await?;
    args.run(keylight).await?;

    Ok(())
}
