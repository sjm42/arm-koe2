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
use embedded_hal::digital::v2::OutputPin;
#[cfg(feature = "blue_pill")]
use hal::gpio::{IOPinSpeed, OutputSpeed};
#[cfg(feature = "blue_pill")]
use stm32f1xx_hal as hal;

// https://www.st.com/en/microcontrollers-microprocessors/stm32f411re.html
#[cfg(feature = "nucleo_f411")]
use crate::hal::gpio::Speed;
#[cfg(feature = "nucleo_f411")]
use stm32f4xx_hal as hal;

use crate::hal::{pac, prelude::*};

// This is needed only for stm32f103 hal

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    let cp = cortex_m::peripheral::Peripherals::take().unwrap();

    let mut delay;
    let mut led;

    // On Nucleo stm32f411 User led LD2 is on PA5, active high
    #[cfg(feature = "nucleo_f411")]
    {
        let pa = dp.GPIOA.split();
        let pc = dp.GPIOC.split();

        // Clock outputs are as alt functions on MCO1=PA8, MCO2=PC9
        let _mco1 = pa.pa8.into_alternate::<0>().set_speed(Speed::VeryHigh);
        let _mco2 = pc.pc9.into_alternate::<0>().set_speed(Speed::VeryHigh);

        // Enable clock outputs 1+2
        // With 100MHz sysclk we should see 8/5 = 1.6 MHz on MCO1 and 100/5 = 20MHz on MCO2
        dp.RCC.cfgr.modify(|_r, w| {
            w.mco1pre().div5();
            w.mco1().hsi();
            w.mco2pre().div5();
            w.mco2().sysclk();
            w
        });

        // Setup system clock to 100 MHz
        let rcc = dp.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(100.mhz()).freeze();

        led = pa.pa5.into_push_pull_output();

        // Create a delay abstraction based on SysTick
        delay = hal::delay::Delay::new(cp.SYST, &clocks);
    }

    // On blue pill stm32f103 user led is on PC13, active low
    #[cfg(feature = "blue_pill")]
    {
        // Enable clock output MCO
        dp.RCC.cfgr.modify(|_r, w| w.mco().sysclk());

        let mut rcc = dp.RCC.constrain();
        let mut pa = dp.GPIOA.split(&mut rcc.apb2);
        let mut pc = dp.GPIOC.split(&mut rcc.apb2);

        // Clock outputs is alt function on MCO=PA8
        let _mco = pa
            .pa8
            .into_alternate_push_pull(&mut pa.crh)
            .set_speed(&mut pa.crh, IOPinSpeed::Mhz50);

        // Setup system clocks
        let mut flash = dp.FLASH.constrain();
        let clocks = rcc
            .cfgr
            .use_hse(8.mhz())
            .sysclk(36.mhz())
            .freeze(&mut flash.acr);

        led = pc.pc13.into_push_pull_output(&mut pc.crh);

        // Create a delay abstraction based on SysTick
        delay = hal::delay::Delay::new(cp.SYST, clocks);
    }

    loop {
        // On blue pill, LED is on with output low
        #[cfg(feature = "blue_pill")]
        let _ = led.set_low();
        #[cfg(feature = "nucleo_f411")]
        led.set_high();

        delay.delay_ms(200_u32);

        #[cfg(feature = "blue_pill")]
        let _ = led.set_high();
        #[cfg(feature = "nucleo_f411")]
        led.set_low();

        delay.delay_ms(800_u32);
    }
}
// EOF
