use bevy::prelude::*;

const DEFAULT_SLOT_ANMOUNTS: u8 = 5;
const SLOT_PLACEHOLDER: EmptySlot = EmptySlot;

pub struct PlayerInventoryPlugin;

/*impl Plugin for PlayerInventorySystem {
	fn build(&self, app: &mut App) {
		app.add_event::<WeaponPickup>()
			.add_system_set(SystemSet::new()
				.after("shoot")
			)
	};
}*/

trait Carry {
	fn after_pickup(&self) -> Self
	where
		Self: Sized;
	fn before_pickup(&self) -> Self
	where
		Self: Sized;
	fn on_use(&self) -> Self
	where
		Self: Sized;
	fn before_drop(&self) -> Self
	where
		Self: Sized;
	fn after_drop(&self) -> Self
	where
		Self: Sized;
}

enum PlayerInventoryError {
	InvalidSlot(u8),
}

struct EmptySlot;

impl Carry for EmptySlot {
	fn after_pickup(&self) -> Self {
		println!("Empty slot picked up");
		Self
	}

	fn before_pickup(&self) -> Self {
		println!("Empty slot will be picked up");
		Self
	}

	fn on_use(&self) -> Self {
		println!("Empty slot can be used");
		Self
	}

	fn before_drop(&self) -> Self {
		println!("Empty Slot about to be Dropped");
		Self
	}

	fn after_drop(&self) -> Self {
		println!("Empty Slot Dropped. Replacing with another empty slot");
		Self
	}
}

pub struct PlayerInventory {
	pub slots: Vec<Box<dyn Carry>>,
	pub slots_amount: u8,
	pub active_slot: u8,
}

impl PlayerInventory {
	fn new_empty() -> Self {
		Self {
			slots_amount: DEFAULT_SLOT_ANMOUNTS,
			slots: vec![Box::new(EmptySlot), Box::new(SLOT_PLACEHOLDER)],
			active_slot: 0,
		}
	}

	fn check_slot(self, slot: usize) -> Result<Box<Self>, PlayerInventoryError> {
		if slot < self.slots_amount as usize {
			Ok(Box::new(self))
		} else {
			Err(PlayerInventoryError::InvalidSlot(slot as u8))
		}
	}

	pub fn set_slot<T>(
		mut self,
		slot: usize,
		content: T,
	) -> Result<Box<Self>, PlayerInventoryError> {
		self.check_slot(slot)?;
		self.slots[slot] = Box::new(content);
		Ok(Box::new(self))
	}

	pub fn get_slot<T>(self, slot: usize) -> Result<T, PlayerInventoryError>
	where
		T: Carry,
	{
		self.check_slot(slot)?;
		self.slots[*slot]
	}

	pub fn drop_slot<T>(self, slot: usize) -> Result<Box<Self>, PlayerInventoryError> {
		self.check_slot(slot)?;
		&self.slots[slot].after_pickup;
		Box::new(*self.slots[slot])
	}

	pub fn clear(self) -> Box<Self> {
		for i in 0..&self.slots.len() {
			self.slots[i] = SLOT_PLACEHOLDER;
		}
		Box::new(self)
	}
}
