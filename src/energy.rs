// First, we are going to introduce some units of energy. For whatever reason, we prefer BTU above
// Joules and Calories, but we want to support all 3 of these in this module. Double check the
// conversion methods, and make sure you fully understand them.

use std::marker::PhantomData;

// You may uncomment and use the following import if you need it. You may also read its
// documentation at https://doc.rust-lang.org/std/cell/struct.RefCell.html
use std::cell::RefCell;

#[derive(Eq, PartialEq, Debug, Clone, Copy)]
pub struct Joule(pub u32);
#[derive(Eq, PartialEq, Debug, Clone, Copy)]
pub struct Calorie(pub u32);

pub type BTU = u32;

impl From<Joule> for BTU {
    fn from(j: Joule) -> Self {
        j.0 / 1055
    }
}

impl From<BTU> for Joule {
    fn from(b: BTU) -> Self {
        Self(b * 1055)
    }
}

impl From<Calorie> for BTU {
    fn from(c: Calorie) -> Self {
        c.0 / 251
    }
}

impl From<BTU> for Calorie {
    fn from(b: BTU) -> Self {
        Calorie(b * 251)
    }
}

// Now, we start defining some types of fuel.

/// A technology for storing energy for later consumption.
pub trait Fuel {
    /// The output unit of the energy density.
    ///
    /// Think about this: why did we chose this to be an associated type rather than a generic?
    type Output: Into<BTU> + From<BTU>;

    /// The amount of energy contained in a single unit of fuel.
    fn energy_density() -> Self::Output;
}

pub struct Diesel;
impl Fuel for Diesel {
    type Output = Joule;
    fn energy_density() -> Self::Output {
        Self::Output::from(100)
    }
}

pub struct LithiumBattery;
impl Fuel for LithiumBattery {
    type Output = Calorie;
    fn energy_density() -> Self::Output {
        Self::Output::from(200)
    }
}

pub struct Uranium;
impl Fuel for Uranium {
    type Output = Joule;
    fn energy_density() -> Self::Output {
        Self::Output::from(1000)
    }
}

/// A container for any fuel type.
pub struct FuelContainer<F: Fuel> {
    /// The amount of fuel.
    amount: u32,
    /// NOTE: Fuel doesn't really have any methods that require `&self` on it,
    /// so any information that we can get, we can get from `F` as **TYPE**, we don't really need
    /// to store an instance of `F`, like `fuel: F` as a struct field. But to satisfy the compiler,
    /// we must use `F` somewhere.
    /// Thus, this is the perfect use case of `PhantomData`.
    _marker: PhantomData<F>,
}

impl<F: Fuel> FuelContainer<F> {
    pub fn new(amount: u32) -> Self {
        Self {
            amount,
            _marker: Default::default(),
        }
    }
}

/// Something that can provide energy from a given `F` fuel type, like a power-plant.
pub trait ProvideEnergy<F: Fuel> {
    /// Consume the fuel container and return the created energy, based on the power density of the
    /// fuel and potentially other factors.
    ///
    /// Some fuel providers might have some kind of decay or inefficiency, which should be reflected
    /// here. Otherwise, [ProvideEnergy::provide_energy_with_efficiency] or
    /// [ProvideEnergy::provide_energy_ideal] might be good enough.
    ///
    /// Not all `ProvideEnergy` implementations need to have internal state. Therefore, this
    /// interface accepts `&self`, not `&mut self`. You might need to use special language features
    /// to overcome this.
    fn provide_energy(&self, f: FuelContainer<F>) -> <F as Fuel>::Output;

    /// Convert the amount of fuel in `f` with an exact efficiency of `e`.
    ///
    /// NOTE: all efficiencies are interpreted as u8 values that can be at most 100, and represent a
    /// percent. If an efficiency above 100 is supplied, the code should treat it as 100. That is to
    /// say that the efficiency is "saturating" at 100%.
    ///
    /// This method must be provided as it will be the same in all implementations.
    fn provide_energy_with_efficiency(&self, f: FuelContainer<F>, e: u8) -> <F as Fuel>::Output {
        let density = F::energy_density().into();
        let energy = f.amount * density;
        if e < 100 {
            return <F as Fuel>::Output::from(energy * e as u32 / 100);
        }
        <F as Fuel>::Output::from(energy)
    }

    /// Same as [`ProvideEnergy::provide_energy_with_efficiency`], but with an efficiency of 100.
    ///
    /// This method must be provided as it will be the same in all implementations.
    fn provide_energy_ideal(&self, f: FuelContainer<F>) -> <F as Fuel>::Output {
        self.provide_energy_with_efficiency(f, 100)
    }
}

/// A nuclear reactor that can only consume `Uranium` and provide energy with 99% efficiency.
pub struct NuclearReactor;
impl<F: Fuel> ProvideEnergy<F> for NuclearReactor {
    fn provide_energy(&self, f: FuelContainer<F>) -> <F as Fuel>::Output {
        let density = Uranium::energy_density();
        let energy = BTU::from(density) * 99 / 100 * f.amount;
        <F as Fuel>::Output::from(energy)
    }
}

/// A combustion engine that can only consume `Diesel`.
///
/// The `DECAY` const must be interpreted as such: per every `DECAY` times `provide_energy` is
/// called on an instance of this type, the efficiency should reduce by one. The initial efficiency
/// must be configurable with a `fn new(efficiency: u8) -> Self`.
pub struct InternalCombustion<const DECAY: u32> {
    pub efficiency: u8,
    pub counter: RefCell<u32>,
}

impl<const DECAY: u32> InternalCombustion<DECAY> {
    pub fn new(efficiency: u8) -> Self {
        Self {
            efficiency,
            counter: RefCell::new(0),
        }
    }
}

impl<const DECAY: u32, F: Fuel> ProvideEnergy<F> for InternalCombustion<DECAY> {
    fn provide_energy(&self, f: FuelContainer<F>) -> <F as Fuel>::Output {
        let counter = self.counter.take();
        let mut efficiency = self.efficiency;
        if efficiency > 100 {
            efficiency = 100;
        }
        if counter % DECAY == 0 {
            efficiency = 100 - (counter / DECAY) as u8;
        }
        let energy = self.provide_energy_with_efficiency(f, efficiency);
        self.counter.replace(counter + 1);
        energy
    }
}

/// A hypothetical device that can, unlike the `InternalCombustion`, consume **any fuel** that's of
/// type `trait Fuel`. It can provide a fixed efficiency regardless of fuel type. As before,
/// EFFICIENCY is a u8 whose value should not exceed 100, is interpreted as a percent, and should
/// saturate at 100% when a higher value is supplied.
pub struct OmniGenerator<const EFFICIENCY: u8>;

// NOTE: implement `ProvideEnergy` for `OmniGenerator` using only one `impl` block.
impl<const EFFICIENCY: u8, F: Fuel> ProvideEnergy<F> for OmniGenerator<EFFICIENCY> {
    fn provide_energy(&self, f: FuelContainer<F>) -> <F as Fuel>::Output {
        self.provide_energy_with_efficiency(f, EFFICIENCY)
    }
}

/// A type that can wrap two different fuel types and mix them together.
///
/// The energy density of the new fuel type is the average of the two given, once converted to BTU.
/// The output unit should also be BTU.
///
/// This can represent a new fuel type, thus it must implement `Fuel`.
pub struct Mixed<F1: Fuel, F2: Fuel>(PhantomData<(F1, F2)>);

impl<F1: Fuel, F2: Fuel> Fuel for Mixed<F1, F2> {
    type Output = BTU;

    fn energy_density() -> Self::Output {
        (F1::energy_density().into() + F2::energy_density().into()) / 2
    }
}

// Now think about how you can make the mixer configurable, such that it would produce a new fuel
// with an energy density that is more influences by one type than the other.
//
// For example, you have a mixer of F1, F2, and some coefficient C1, where the energy density of the
// mixture is `F1 * C1 + F2 * (1 - C1) )` where `C1` is a ratio (which you have to represent again
// with a u8 percent).
//
// The main trick is to overcome the fact that `fn energy_density` does not take in a `self`, so the
// coefficients need to be incorporated in some other way (you've already seen examples of that in
// this file ;)).
pub struct CustomMixed<const C: u8, F1, F2>(PhantomData<(F1, F2)>);
impl<const C: u8, F1: Fuel, F2: Fuel> Fuel for CustomMixed<C, F1, F2> {
    type Output = BTU;

    fn energy_density() -> Self::Output {
        let mix_density = (F1::energy_density().into() * C as u32
            + F2::energy_density().into() * (100 - C as u32))
            / 100;
        mix_density
    }
}

// Now, any of our existing energy providers can be used with a mix fuel.

/// A function that returns the energy produced by the `OmniGenerator` with efficiency of 80%, when
/// the fuel type is an even a mix of `Diesel` as `LithiumBattery`;
pub fn omni_80_energy(amount: u32) -> BTU {
    let mx_density = Mixed::<Diesel, LithiumBattery>::energy_density();
    let energy = amount * mx_density * 80 / 100;
    energy
}

// Finally, let's consider marker traits, and some trait bounds.

/// Some traits are just markers. They don't bring any additional functionality anything, other than
/// marking a type with some trait.
pub trait IsRenewable {}
impl IsRenewable for LithiumBattery {}

/// Define the following struct such that it only provides energy if the fuel is `IsRenewable`.
///
/// It has perfect efficiency.
pub struct GreenEngine<F: Fuel + IsRenewable>(pub PhantomData<F>);
impl<F: Fuel + IsRenewable> ProvideEnergy<F> for GreenEngine<F> {
    fn provide_energy(&self, f: FuelContainer<F>) -> <F as Fuel>::Output {
        self.provide_energy_ideal(f)
    }
}

/// Define the following struct such that it only provides energy if the fuel's output type is
/// `BTU`.
///
/// It has perfect efficiency.
pub struct BritishEngine<F: Fuel>(pub PhantomData<F>);
impl<F: Fuel> ProvideEnergy<F> for BritishEngine<F> {
    fn provide_energy(&self, f: FuelContainer<F>) -> <F as Fuel>::Output {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    trait ToBTU {
        fn to_btu(self) -> BTU;
    }

    impl<T: Into<BTU>> ToBTU for T {
        fn to_btu(self) -> BTU {
            self.into()
        }
    }

    #[test]
    fn nuclear() {
        let nr = NuclearReactor;
        assert_eq!(
            nr.provide_energy(FuelContainer::<Uranium>::new(10))
                .to_btu(),
            9900
        );
        assert_eq!(
            nr.provide_energy(FuelContainer::<Uranium>::new(10))
                .to_btu(),
            9900
        );
    }

    #[test]
    fn ic_1() {
        let ic = InternalCombustion::<3>::new(120);
        assert_eq!(
            ic.provide_energy(FuelContainer::<Diesel>::new(10)).to_btu(),
            1000
        );
        assert_eq!(
            ic.provide_energy(FuelContainer::<Diesel>::new(10)).to_btu(),
            1000
        );
        assert_eq!(
            ic.provide_energy(FuelContainer::<Diesel>::new(10)).to_btu(),
            1000
        );
        assert_eq!(
            ic.provide_energy(FuelContainer::<Diesel>::new(10)).to_btu(),
            990
        );
    }

    #[test]
    fn omni_1() {
        let og = OmniGenerator::<100>;
        assert_eq!(
            og.provide_energy(FuelContainer::<Uranium>::new(10))
                .to_btu(),
            10000
        );
        assert_eq!(
            og.provide_energy(FuelContainer::<Diesel>::new(10)).to_btu(),
            1000
        );
        assert_eq!(
            og.provide_energy(FuelContainer::<LithiumBattery>::new(10))
                .to_btu(),
            2000
        );
    }

    #[test]
    fn mixed_1() {
        assert_eq!(
            Mixed::<Diesel, LithiumBattery>::energy_density().to_btu(),
            150
        );
    }

    #[test]
    fn custom_mixed_1() {
        // custom with 50 is the same as Mixed.
        assert_eq!(
            CustomMixed::<50, Diesel, LithiumBattery>::energy_density().to_btu(),
            Mixed::<Diesel, LithiumBattery>::energy_density()
        );
    }

    #[test]
    fn custom_mixed_2() {
        // custom with 80 is the same as Mixed.
        assert_eq!(
            CustomMixed::<20, Diesel, LithiumBattery>::energy_density().to_btu(),
            180
        );
    }

    #[test]
    fn omni_80() {
        assert_eq!(omni_80_energy(10), 1200);
    }

    #[test]
    fn green_should_work() {
        let ge = GreenEngine::<LithiumBattery>(PhantomData::<LithiumBattery>);
        assert_eq!(
            ge.provide_energy(FuelContainer::<LithiumBattery>::new(10))
                .to_btu(),
            2000
        );
    }

    //     /// Define the following struct such that it only provides energy if the fuel's output type is
    // /// `BTU`.
    // ///
    // /// It has perfect efficiency.
    // pub struct BritishEngine<F: Fuel + From<Joule> + From<Calorie>>(pub PhantomData<F>);
    // impl<F: Fuel + From<Joule> + From<Calorie>> ProvideEnergy<F> for BritishEngine<F> {
    //     fn provide_energy(&self, f: FuelContainer<F>) -> <F as Fuel>::Output {
    //         self.provide_energy_ideal(f)
    //     }
    // }

    #[test]
    fn british_should_work() {
        todo!()
    }
}
