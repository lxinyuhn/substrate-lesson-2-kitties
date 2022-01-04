#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.io/docs/en/knowledgebase/runtime/frame>
pub use pallet::*;

mod mock;
mod tests;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use frame_support::{
		sp_runtime::traits::{AtLeast32BitUnsigned, Bounded, CheckedAdd, One},
		traits::{ Randomness, Currency, ReservableCurrency, tokens::ExistenceRequirement },
	};
	use sp_io::hashing::blake2_128;
	use scale_info::TypeInfo;

	#[cfg(feature = "std")]
	use frame_support::serde::{Deserialize, Serialize};

	// type AccountOf<T> = <T as frame_system::Config>::AccountId;
	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;


	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct Kitty<T: Config> {
		pub dna: [u8; 16],
		pub price: Option<BalanceOf<T>>,
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	

	// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
		type Randomness: Randomness<Self::Hash, Self::BlockNumber>;
		type ReservePerKitty: Get<BalanceOf<Self>>;
		type KittyIndex: Parameter + AtLeast32BitUnsigned + Default + Copy + Bounded + CheckedAdd;
	}

	// Errors.
	#[pallet::error]
	pub enum Error<T> {
		KittiesCountOverflow,
		NotOwner,
		SameParentIndex,
		KittyNotExist,
		KittyBidPriceTooLow,
		NotEnoughBalance,
		KittyNotForSale,
		OwnerCanNotBuy,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		KittyCreate(T::AccountId, T::KittyIndex),
		KittyTransfered(T::AccountId, T::AccountId, T::KittyIndex),
		PriceChange(T::AccountId, T::KittyIndex),
		Bought(T::AccountId, T::AccountId, T::KittyIndex, BalanceOf<T>),
		KittyBreed(T::AccountId, T::KittyIndex),
	}

	// Storage items.

	#[pallet::storage]
	#[pallet::getter(fn kitties_count)]
	pub type KittiesCount<T: Config> = StorageValue<_, T::KittyIndex>;

	#[pallet::storage]
	#[pallet::getter(fn kitties)]
	pub type Kitties<T: Config> = StorageMap<_, Blake2_128Concat, T::KittyIndex, Kitty<T>>;

	#[pallet::storage]
	#[pallet::getter(fn owner)]
	pub type Owner<T:Config> = StorageMap<_, Blake2_128Concat, T::KittyIndex, Option<T::AccountId>, ValueQuery>;

	// Our pallet's genesis configuration.
	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub kitties: Vec<(T::AccountId, [u8; 16])>,
	}

	// Required to implement default for GenesisConfig.
	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> GenesisConfig<T> {
			GenesisConfig { kitties: vec![] }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			for (acct, dna) in &self.kitties {
				let _ = <Pallet<T>>::mint(acct, Some(dna.clone()));
			}
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]
		pub fn create(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			let amount: BalanceOf<T> = T::ReservePerKitty::get().into();
			T::Currency::reserve(&who, amount).map_err(|_| Error::<T>::NotEnoughBalance)?;

			let kitty_id = Self::mint(&who, None)?;
			
			Self::deposit_event(Event::KittyCreate(who, kitty_id));
			Ok(().into())
		}

		#[pallet::weight(0)]
		pub fn transfer(
			origin: OriginFor<T>, 
			to: T::AccountId,			
			kitty_id: T::KittyIndex
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			let mut kitty = Self::kitties(&kitty_id).ok_or(<Error<T>>::KittyNotExist)?;
            ensure!(Some(who.clone()) == Owner::<T>::get(kitty_id), Error::<T>::NotOwner);
			
			kitty.price = None;

            Owner::<T>::insert(
                kitty_id,
				Some(&to)
            );			
			Self::deposit_event(Event::KittyTransfered(who, to, kitty_id));
			Ok(().into())
		}   	
		
		#[pallet::weight(0)]
		pub fn breed(
			origin: OriginFor<T>, 		
			kitty_id_1: T::KittyIndex,
			kitty_id_2: T::KittyIndex,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
            ensure!(kitty_id_1!= kitty_id_2, Error::<T>::SameParentIndex);
			let kitty1= Self::kitties(kitty_id_1).ok_or(Error::<T>::KittyNotExist)?;
			let kitty2= Self::kitties(kitty_id_2).ok_or(Error::<T>::KittyNotExist)?;

			let dna1 = kitty1.dna;
			let dna2 = kitty2.dna;

			let selector = Self::random_value();
			let mut new_dna = [0u8; 16];
			for i in 0..dna1.len(){
				new_dna[i] = (selector[i] & dna1[i]) | (!selector[i] & dna2[i]);
			}

			let kitty_id = Self::mint(&who, Some(new_dna))?;
			

			Self::deposit_event(Event::KittyBreed(who, kitty_id));

			Ok(().into())
		}  		


		#[pallet::weight(0)]
		pub fn set_price(
			origin: OriginFor<T>, 		
			kitty_id: T::KittyIndex,
			new_price: Option<BalanceOf<T>>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let mut kitty = Self::kitties(&kitty_id).ok_or(<Error<T>>::KittyNotExist)?;
			ensure!(Some(who.clone()) == Owner::<T>::get(kitty_id), Error::<T>::NotOwner);
			kitty.price = new_price.clone();
			Kitties::<T>::insert(kitty_id, kitty);
			Self::deposit_event(Event::PriceChange(who, kitty_id));
			Ok(().into())
		}

		#[pallet::weight(0)]
		pub fn buy(
			origin: OriginFor<T>, 		
			kitty_id: T::KittyIndex,
			bid_price: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			ensure!(T::Currency::free_balance(&who) >= bid_price, <Error<T>>::NotEnoughBalance);
			let mut kitty= Self::kitties(kitty_id).ok_or(Error::<T>::KittyNotExist)?;
            ensure!(Some(who.clone()) != Owner::<T>::get(kitty_id), Error::<T>::OwnerCanNotBuy);			

			if let Some(ask_price) = kitty.price {
				ensure!(ask_price <= bid_price, <Error<T>>::KittyBidPriceTooLow);
			} else {
				Err(<Error<T>>::KittyNotForSale)?;
			}
			
			if let Some(seller) = Owner::<T>::get(kitty_id){
				T::Currency::transfer(&who, &seller, bid_price, ExistenceRequirement::KeepAlive)?;
				Owner::<T>::insert(kitty_id, Some(who.clone()));

				kitty.price = None;
				Kitties::<T>::insert(kitty_id, kitty);
				Self::deposit_event(Event::Bought(who, seller, kitty_id, bid_price));				
			}	

			Ok(().into())
		}		
	}

	//** Our helper functions.**//

	impl<T: Config> Pallet<T>{

		// Helper to mint a Kitty.
		pub fn mint(
			owner: &T::AccountId,
			dna: Option<[u8; 16]>
		) -> Result<T::KittyIndex, Error<T>> {
			let kitty = Kitty::<T> {
				dna: dna.unwrap_or_else(Self::random_value),
				price: None,
			};

			let kitty_id = match Self::kitties_count(){
				Some(id) =>{
					id.checked_add(&One::one()).ok_or(<Error<T>>::KittiesCountOverflow)?
				},
				None => {
					One::one()
				}
			};			

			Kitties::<T>::insert(kitty_id, kitty);
			Owner::<T>::insert(kitty_id, Some(owner.clone()));
			KittiesCount::<T>::put(kitty_id);

			Ok(kitty_id)
		}		

		fn random_value() -> [u8;16]{
			let payload = (
				T::Randomness::random_seed(),
				<frame_system::Pallet<T>>::extrinsic_index(),
			);
			payload.using_encoded(blake2_128)
		}
	}		
}
