// main.rs

#![no_std]
#![no_main]
#![deny(unsafe_code)]
#![deny(warnings)]

// use cortex_m::asm;
use cortex_m_rt::entry;
use panic_halt as _;

// https://www.st.com/en/microcontrollers-microprocessors/stm32f103.html
#[cfg(feature = "blue_pill")]
use stm32f1xx_hal as hal;

// https://www.st.com/en/microcontrollers-microprocessors/stm32f411re.html
#[cfg(feature = "nucleo_f411")]
use stm32f4xx_hal as hal;

use crate::hal::{pac, prelude::*};

// This is needed only for stm32f103 hal
#[cfg(feature = "blue_pill")]
use embedded_hal::digital::v2::OutputPin;

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    let cp = cortex_m::peripheral::Peripherals::take().unwrap();

    let mut delay;
    let mut led;

    // On Nucleo stm32f411 User led LD2 is on PA5
    #[cfg(feature = "nucleo_f411")]
    {
        let rcc = dp.RCC.constrain();
        let pa = dp.GPIOA.split();
        let _pc = dp.GPIOC.split();

        // TODO: enable clock outputs!

        // Clock outputs are as alt functions on MCO1=PA8, MCO2=PC9
        // let _mco1 = pa.pa8.into_alternate::<AF1>().set_speed(Speed::VeryHigh);
        // let _mco2 = pc.pc9.into_alternate::<AF1>().set_speed(Speed::VeryHigh);

        // Setup system clock to 100 MHz
        let clocks = rcc.cfgr.sysclk(100.mhz()).freeze();

        // Create a delay abstraction based on SysTick
        delay = hal::delay::Delay::new(cp.SYST, &clocks);

        // Setup PA5 as push-pull output

        led = pa.pa5.into_push_pull_output();
    }

    // On blue pill stm32f103 user led is on PC13
    #[cfg(feature = "blue_pill")]
    {
        // Setup system clock to 72 MHz
        let mut rcc = dp.RCC.constrain();
        let mut pc = dp.GPIOC.split(&mut rcc.apb2);

        // Setup PC13 as push-pull output
        led = pc.pc13.into_push_pull_output(&mut pc.crh);

        // Setup system clock to 72 MHz
        let mut flash = dp.FLASH.constrain();
        let clocks = rcc.cfgr.sysclk(72.mhz()).freeze(&mut flash.acr);

        // Create a delay abstraction based on SysTick
        delay = hal::delay::Delay::new(cp.SYST, clocks);
    }

    loop {
        let _ = led.set_high();
        delay.delay_ms(200_u32);

        let _ = led.set_low();
        delay.delay_ms(1300_u32);
    }
}

// EOF
