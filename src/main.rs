#![no_std]
#![no_main]

use defmt::println;
use defmt_rtt as _;
use panic_probe as _;

use cortex_m_rt::entry;
use stm32f1xx_hal::{
    flash::FlashExt,
    pac::{self},
    prelude::*,
    rcc::{Config, RccExt},
    time::Hertz,
    timer::{Tim2NoRemap, Timer},
};

#[entry]
fn main() -> ! {
    defmt::println!("STM32F103C8 PWM");

    let dp = pac::Peripherals::take().unwrap();

    let cp = cortex_m::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();

    let rcc = dp.RCC.constrain();

    let clock_config = Config::default()
        .use_hse(Hertz::MHz(8))
        .sysclk(Hertz::MHz(72))
        .hclk(Hertz::MHz(72))
        .pclk1(Hertz::MHz(36));

    let mut clocks = rcc.freeze(clock_config, &mut flash.acr);

    let mut gpioa = dp.GPIOA.split(&mut clocks);

    let pins = (
        gpioa.pa0.into_alternate_push_pull(&mut gpioa.crl),
        gpioa.pa2.into_alternate_push_pull(&mut gpioa.crl),
    );

    let mut afio = dp.AFIO.constrain(&mut clocks);

    let timer2 = Timer::new(dp.TIM2, &mut clocks);

    let mut pwm = timer2.pwm_hz::<Tim2NoRemap, _, _>(pins, &mut afio.mapr, Hertz::kHz(1));

    pwm.enable(stm32f1xx_hal::timer::Channel::C1);
    pwm.enable(stm32f1xx_hal::timer::Channel::C3);

    let max_duty = pwm.get_max_duty();

    println!("max duty: {}", max_duty);

    // return pins tupple
    let pwm_channels = pwm.split();

    let mut pwm_pa0 = pwm_channels.0;
    let mut pwm_pa2 = pwm_channels.1;

    let mut delay = Timer::syst_external(cp.SYST, &clocks.clocks).delay();

    pwm_pa0.set_duty(max_duty);
    pwm_pa2.set_duty(0);
    delay.delay_ms(1000u16);

    // duty cycle 50%
    defmt::println!("duty cycle 50%");
    pwm_pa0.set_duty(max_duty / 2);
    pwm_pa2.set_duty(max_duty / 2);
    delay.delay_ms(1000u16);

    pwm_pa0.set_duty(0);
    pwm_pa2.set_duty(max_duty);
    delay.delay_ms(1000u16);

    loop {
        for duty in 0..=max_duty {
            pwm_pa0.set_duty(duty);
            pwm_pa2.set_duty(max_duty - duty);
            delay.delay_us(100u8);
        }

        for duty in 0..=max_duty {
            pwm_pa0.set_duty(max_duty - duty);
            pwm_pa2.set_duty(duty);
            delay.delay_us(100u8);
        }
    }
}
