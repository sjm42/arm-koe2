// main.rs

#![no_std]
#![no_main]
#![deny(unsafe_code)]
#![allow(unused_mut)]
#![deny(warnings)]

use cortex_m_rt::entry;
use panic_halt as _;

// Supported hardware:
//
// BluePill
// https://www.st.com/en/microcontrollers-microprocessors/stm32f103.html
//
// BlackPill
// https://www.st.com/en/microcontrollers-microprocessors/stm32f411ce.html
//
// Nucleo f411
// https://www.st.com/en/microcontrollers-microprocessors/stm32f411re.html
//

#[cfg(feature = "blue_pill")]
use stm32f1xx_hal as hal;

#[cfg(any(feature = "black_pill", feature = "nucleo_f411"))]
use stm32f4xx_hal as hal;

use crate::hal::{gpio::*, pac, prelude::*};

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

fn init() -> (ErasedPin<Output<PushPull>>, hal::delay::Delay) {
    let dp = pac::Peripherals::take().unwrap();
    let cp = cortex_m::peripheral::Peripherals::take().unwrap();

    // Enable clock output MCO
    #[cfg(feature = "blue_pill")]
    dp.RCC.cfgr.modify(|_r, w| w.mco().sysclk());

    // Enable clock outputs 1+2
    // With 100MHz sysclk we should see 8/5 = 1.6 MHz on MCO1 and 100/5 = 20MHz on MCO2
    #[cfg(any(feature = "black_pill", feature = "nucleo_f411"))]
    dp.RCC.cfgr.modify(|_r, w| {
        w.mco1pre().div5();
        w.mco1().hsi();
        w.mco2pre().div5();
        w.mco2().sysclk();
        w
    });

    let rcc = dp.RCC.constrain();
    let mut pa = dp.GPIOA.split();
    let mut pc = dp.GPIOC.split();
    #[cfg(feature = "blue_pill")]
    let mut flash = dp.FLASH.constrain();

    // Setup system clocks
    #[cfg(feature = "blue_pill")]
    let clocks = rcc
        .cfgr
        .use_hse(8.mhz()) // Use High Speed External 8Mhz crystal oscillator
        .sysclk(72.mhz()) // Use the PLL to multiply SYSCLK to 72MHz
        .hclk(72.mhz()) // Leave AHB prescaler at /1
        .pclk1(36.mhz()) // Use the APB1 prescaler to divide the clock to 36MHz (max supported)
        .pclk2(72.mhz()) // Leave the APB2 prescaler at /1
        .adcclk(12.mhz()) // ADC prescaler of /6 (max speed of 14MHz, but /4 gives 18MHz)
        .freeze(&mut flash.acr);

    // Setup system clock to 100 MHz
    #[cfg(any(feature = "black_pill", feature = "nucleo_f411"))]
    let clocks = rcc.cfgr.sysclk(100.mhz()).freeze();

    // Clock outputs is alt function on MCO=PA8
    #[cfg(feature = "blue_pill")]
    let _mco = pa
        .pa8
        .into_alternate_push_pull(&mut pa.crh)
        .set_speed(&mut pa.crh, IOPinSpeed::Mhz50);

    // Clock outputs are as alt functions on MCO1=PA8, MCO2=PC9
    #[cfg(any(feature = "black_pill", feature = "nucleo_f411"))]
    let _mco1 = pa.pa8.into_alternate::<0>().set_speed(Speed::VeryHigh);
    #[cfg(any(feature = "black_pill", feature = "nucleo_f411"))]
    let _mco2 = pc.pc9.into_alternate::<0>().set_speed(Speed::VeryHigh);

    // On Bluepill stm32f103 user led is on PC13, active low
    // Configure gpio C pin 13 as a push-pull output. The `crh` register is passed to the function
    // in order to configure the port. For pins 0-7, crl should be passed instead.
    #[cfg(feature = "blue_pill")]
    let led = pc.pc13.into_push_pull_output(&mut pc.crh).erase();

    // On Blackpill stm32f411 user led is on PC13, active low
    #[cfg(feature = "black_pill")]
    let led = pc.pc13.into_push_pull_output().erase();

    // On Nucleo stm32f411 User led LD2 is on PA5, active high
    #[cfg(feature = "nucleo_f411")]
    let led = pa.pa5.into_push_pull_output().erase();

    // Create a delay abstraction based on SysTick
    #[cfg(feature = "blue_pill")]
    let delay = hal::delay::Delay::new(cp.SYST, clocks);
    #[cfg(any(feature = "black_pill", feature = "nucleo_f411"))]
    let delay = hal::delay::Delay::new(cp.SYST, &clocks);

    (led, delay)
}

fn set_led(led: &mut ErasedPin<Output<PushPull>>, state: bool) {
    #[cfg(any(feature = "black_pill", feature = "blue_pill"))]
    let active_low = true;
    #[cfg(feature = "nucleo_f411")]
    let active_low = false;

    if state ^ active_low {
        led.set_high();
    } else {
        led.set_low();
    }
}
// EOF
