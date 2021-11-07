// main.rs

#![no_std]
#![no_main]
#![deny(unsafe_code)]
#![deny(warnings)]

use cortex_m_rt::entry;
use panic_halt as _;

// https://www.st.com/en/microcontrollers-microprocessors/stm32f103.html
#[cfg(feature = "blue_pill")]
use stm32f1xx_hal as hal;

// https://www.st.com/en/microcontrollers-microprocessors/stm32f411re.html
#[cfg(feature = "nucleo_f411")]
use stm32f4xx_hal as hal;

#[cfg(feature = "nucleo_f411")]
use crate::hal::gpio::Speed;

use crate::hal::{gpio::*, pac, prelude::*};
use embedded_hal::digital::v2::OutputPin;

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

        led = pa.pa5.into_push_pull_output().erase();

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
            .use_hse(8.mhz()) // Use High Speed External 8Mhz crystal oscillator
            .sysclk(72.mhz()) // Use the PLL to multiply SYSCLK to 72MHz
            .hclk(72.mhz()) // Leave AHB prescaler at /1
            .pclk1(36.mhz()) // Use the APB1 prescaler to divide the clock to 36MHz (max supported)
            .pclk2(72.mhz()) // Leave the APB2 prescaler at /1
            .adcclk(12.mhz()) // ADC prescaler of /6 (max speed of 14MHz, but /4 gives 18MHz)
            .freeze(&mut flash.acr);

        // Configure gpio C pin 13 as a push-pull output. The `crh` register is passed to the function
        // in order to configure the port. For pins 0-7, crl should be passed instead.
        led = pc.pc13.into_push_pull_output(&mut pc.crh).downgrade();
        // let led2 = pa.pa9.into_push_pull_output(&mut pa.crh).downgrade();

        // Create a delay abstraction based on SysTick
        delay = hal::delay::Delay::new(cp.SYST, clocks);
    }

    loop {
        setled(&mut led, true);
        delay.delay_ms(200_u32);

        setled(&mut led, false);
        delay.delay_ms(800_u32);
    }
}

#[cfg(feature = "blue_pill")]
fn setled(led: &mut Pxx<Output<PushPull>>, state: bool) {
    // On blue pill, LED is on with output low
    if state {
        let _ = led.set_low();
    } else {
        let _ = led.set_high();
    }
}

#[cfg(feature = "nucleo_f411")]
fn setled(led: &mut dyn OutputPin<Error = core::convert::Infallible>, state: bool) {
    if state {
        let _ = led.set_high();
    } else {
        let _ = led.set_low();
    }
}

// EOF
