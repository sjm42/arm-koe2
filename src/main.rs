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
    let (mut led, mut delay) = init();

    loop {
        for _i in 1..=3 {
            set_led(&mut led, true);
            delay.delay_ms(100_u32);

            set_led(&mut led, false);
            delay.delay_ms(400_u32);
        }
        delay.delay_ms(1000_u32);
    }
}

#[cfg(feature = "blue_pill")]
fn init() -> (Pxx<Output<PushPull>>, hal::delay::Delay) {
    let dp = pac::Peripherals::take().unwrap();
    let cp = cortex_m::peripheral::Peripherals::take().unwrap();

    // Enable clock output MCO
    dp.RCC.cfgr.modify(|_r, w| w.mco().sysclk());

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();
    let mut pa = dp.GPIOA.split(&mut rcc.apb2);
    let mut pc = dp.GPIOC.split(&mut rcc.apb2);

    // Setup system clocks
    let clocks = rcc
        .cfgr
        .use_hse(8.mhz()) // Use High Speed External 8Mhz crystal oscillator
        .sysclk(72.mhz()) // Use the PLL to multiply SYSCLK to 72MHz
        .hclk(72.mhz()) // Leave AHB prescaler at /1
        .pclk1(36.mhz()) // Use the APB1 prescaler to divide the clock to 36MHz (max supported)
        .pclk2(72.mhz()) // Leave the APB2 prescaler at /1
        .adcclk(12.mhz()) // ADC prescaler of /6 (max speed of 14MHz, but /4 gives 18MHz)
        .freeze(&mut flash.acr);

    // Clock outputs is alt function on MCO=PA8
    let _mco = pa
        .pa8
        .into_alternate_push_pull(&mut pa.crh)
        .set_speed(&mut pa.crh, IOPinSpeed::Mhz50);

    // On blue pill stm32f103 user led is on PC13, active low
    // Configure gpio C pin 13 as a push-pull output. The `crh` register is passed to the function
    // in order to configure the port. For pins 0-7, crl should be passed instead.
    let led = pc.pc13.into_push_pull_output(&mut pc.crh).downgrade();

    // Create a delay abstraction based on SysTick
    let delay = hal::delay::Delay::new(cp.SYST, clocks);

    (led, delay)
}

#[cfg(feature = "blue_pill")]
fn set_led(led: &mut Pxx<Output<PushPull>>, state: bool) {
    // On blue pill, LED is on with output low
    if state {
        let _ = led.set_low();
    } else {
        let _ = led.set_high();
    }
}

#[cfg(feature = "nucleo_f411")]
fn init() -> (ErasedPin<Output<PushPull>>, hal::delay::Delay) {
    let dp = pac::Peripherals::take().unwrap();
    let cp = cortex_m::peripheral::Peripherals::take().unwrap();

    // Enable clock outputs 1+2
    // With 100MHz sysclk we should see 8/5 = 1.6 MHz on MCO1 and 100/5 = 20MHz on MCO2
    dp.RCC.cfgr.modify(|_r, w| {
        w.mco1pre().div5();
        w.mco1().hsi();
        w.mco2pre().div5();
        w.mco2().sysclk();
        w
    });

    let rcc = dp.RCC.constrain();
    let pa = dp.GPIOA.split();
    let pc = dp.GPIOC.split();

    // Setup system clock to 100 MHz
    let clocks = rcc.cfgr.sysclk(100.mhz()).freeze();

    // Clock outputs are as alt functions on MCO1=PA8, MCO2=PC9
    let _mco1 = pa.pa8.into_alternate::<0>().set_speed(Speed::VeryHigh);
    let _mco2 = pc.pc9.into_alternate::<0>().set_speed(Speed::VeryHigh);

    // On Nucleo stm32f411 User led LD2 is on PA5, active high
    let led = pa.pa5.into_push_pull_output().erase();

    // Create a delay abstraction based on SysTick
    let delay = hal::delay::Delay::new(cp.SYST, &clocks);

    (led, delay)
}

#[cfg(feature = "nucleo_f411")]
fn set_led(led: &mut dyn OutputPin<Error = core::convert::Infallible>, state: bool) {
    if state {
        let _ = led.set_high();
    } else {
        let _ = led.set_low();
    }
}

// EOF
