#![no_std]
#![no_main]

use core::cell::RefCell;
use cortex_m::interrupt::Mutex;
use embedded_hal::digital::v2::ToggleableOutputPin;
use embedded_time::duration::Microseconds;
use panic_halt as _;
use sparkfun_thing_plus_rp2040::{
    entry,
    hal::{
        self, gpio,
        timer::{Alarm, Alarm0, Timer},
    },
    pac::{self, interrupt, interrupt::TIMER_IRQ_0},
    Pins, XOSC_CRYSTAL_FREQ,
};

type LedPin = gpio::Pin<gpio::bank0::Gpio25, gpio::PushPullOutput>;
type LedAndAlarm = (LedPin, Alarm0);

// This is how we transfer our peripherals into the interrupt handler
static GLOBALS: Mutex<RefCell<Option<LedAndAlarm>>> = Mutex::new(RefCell::new(None));

const DELAY: Microseconds = Microseconds(1_000_000);

#[entry]
fn main() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();
    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

    let _clocks = hal::clocks::init_clocks_and_plls(
        XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    );

    // The single-cycle I/O block controls the GPIO pins
    let sio = hal::Sio::new(pac.SIO);

    // Set the pins to their default state
    let pins = Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let led = pins.led.into_mode();

    let mut timer = Timer::new(pac.TIMER, &mut pac.RESETS);
    let mut alarm = timer.alarm_0().unwrap();

    alarm.schedule(DELAY).unwrap();
    alarm.enable_interrupt();

    // Give away our configured peripherals by moving them into `GLOBALS`
    cortex_m::interrupt::free(|cs| {
        GLOBALS.borrow(cs).replace(Some((led, alarm)));
    });

    unsafe {
        pac::NVIC::unmask(TIMER_IRQ_0);
    }

    loop {
        cortex_m::asm::wfi(); // wait for interrupt
    }
}

#[allow(non_snake_case)]
#[interrupt]
fn TIMER_IRQ_0() {
    static mut LED_AND_ALARM: Option<LedAndAlarm> = None;

    // This is one-time lazy initialization. We steal the variables given to us
    // via `GLOBALS`.
    if LED_AND_ALARM.is_none() {
        cortex_m::interrupt::free(|cs| {
            *LED_AND_ALARM = GLOBALS.borrow(cs).take();
        });
    }

    if let Some((led, alarm)) = LED_AND_ALARM {
        // toggle can't fail, but the embedded-hal traits always allow for it
        let _ = led.toggle();

        // The interrupt doesn't clear itself, so clear it and schedule the
        // next alarm
        alarm.clear_interrupt();
        alarm.schedule(DELAY).unwrap();
    }
}
