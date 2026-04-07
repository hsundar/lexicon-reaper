use serde::{Deserialize, Serialize};

use super::equipment::{Book, BookKind, Potion, Weapon, WeaponKind};
use super::player::Player;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShopItem {
    BuyWeapon(WeaponKind),
    UpgradeWeapon,
    BuyBook(BookKind),
    BuyPotion(Potion),
    UpgradeHp,
    UpgradeAttack,
    UpgradeArmor,
    UpgradeDodge,
    UpgradePotionCapacity,
}

impl ShopItem {
    pub fn name(&self) -> String {
        match self {
            ShopItem::BuyWeapon(kind) => {
                let w = Weapon { kind: kind.clone(), level: 1 };
                format!("Buy {}", w.name())
            }
            ShopItem::UpgradeWeapon => "Upgrade Weapon".to_string(),
            ShopItem::BuyBook(kind) => {
                let b = Book::new(kind.clone());
                format!("Buy {}", b.name())
            }
            ShopItem::BuyPotion(p) => format!("Buy {}", p.name()),
            ShopItem::UpgradeHp => "Max HP +10".to_string(),
            ShopItem::UpgradeAttack => "Attack +2".to_string(),
            ShopItem::UpgradeArmor => "Armor +1".to_string(),
            ShopItem::UpgradeDodge => "Dodge +2%".to_string(),
            ShopItem::UpgradePotionCapacity => "Potion Slots +1".to_string(),
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            ShopItem::BuyWeapon(WeaponKind::BasicScythe) => "Basic weapon, +dodge chance",
            ShopItem::BuyWeapon(WeaponKind::VenomScythe) => "Bonus from poison tiles",
            ShopItem::BuyWeapon(WeaponKind::CrystalScythe) => "Higher crystal tile spawn",
            ShopItem::BuyWeapon(WeaponKind::ThunderScythe) => "High damage + crit bonus",
            ShopItem::BuyWeapon(WeaponKind::VampireScythe) => "Lifesteal on kills",
            ShopItem::BuyWeapon(WeaponKind::GiantScythe) => "Huge bonus for 6+ letter words",
            ShopItem::UpgradeWeapon => "Increase weapon level by 1",
            ShopItem::BuyBook(BookKind::Strength) => "+flat damage per word",
            ShopItem::BuyBook(BookKind::Wisdom) => "+bonus for rare letters",
            ShopItem::BuyBook(BookKind::Health) => "+max HP each stage",
            ShopItem::BuyBook(BookKind::Speed) => "Chance for free extra turn",
            ShopItem::BuyBook(BookKind::Purity) => "Chance to cleanse bad tiles",
            ShopItem::BuyBook(BookKind::Wealth) => "+gem drops from enemies",
            ShopItem::BuyPotion(_) => "Single-use consumable",
            ShopItem::UpgradeHp => "Permanently increase max HP",
            ShopItem::UpgradeAttack => "Permanently increase base attack",
            ShopItem::UpgradeArmor => "Permanently reduce damage taken",
            ShopItem::UpgradeDodge => "Permanently increase dodge chance",
            ShopItem::UpgradePotionCapacity => "Carry one more potion",
        }
    }
}

pub fn cost_for_item(item: &ShopItem, player: &Player) -> u64 {
    match item {
        ShopItem::BuyWeapon(kind) => Weapon::buy_cost(kind),
        ShopItem::UpgradeWeapon => player.weapon.upgrade_cost(),
        ShopItem::BuyBook(kind) => Book::buy_cost(kind),
        ShopItem::BuyPotion(p) => p.buy_cost(),
        ShopItem::UpgradeHp => 80 + 20 * player.upgrades.hp_level as u64,
        ShopItem::UpgradeAttack => 100 + 30 * player.upgrades.attack_level as u64,
        ShopItem::UpgradeArmor => 120 + 40 * player.upgrades.armor_level as u64,
        ShopItem::UpgradeDodge => 150 + 50 * player.upgrades.dodge_level as u64,
        ShopItem::UpgradePotionCapacity => 100 + 80 * player.upgrades.potion_capacity_level as u64,
    }
}

pub fn can_buy(item: &ShopItem, player: &Player) -> bool {
    let cost = cost_for_item(item, player);
    if player.gems < cost {
        return false;
    }
    match item {
        ShopItem::BuyPotion(_) => player.potions.len() < player.max_potions,
        ShopItem::BuyBook(_) => player.books.len() < 3,
        ShopItem::UpgradeWeapon => player.weapon.level < 10,
        ShopItem::UpgradeDodge => player.upgrades.dodge_level < 5,
        ShopItem::UpgradePotionCapacity => player.upgrades.potion_capacity_level < 3,
        _ => true,
    }
}

pub fn buy_item(item: &ShopItem, player: &mut Player) -> Option<String> {
    let cost = cost_for_item(item, player);
    if !can_buy(item, player) {
        return None;
    }

    player.gems -= cost;

    match item {
        ShopItem::BuyWeapon(kind) => {
            player.weapon = Weapon { kind: kind.clone(), level: 1 };
            Some(format!("Equipped {}!", player.weapon.name()))
        }
        ShopItem::UpgradeWeapon => {
            player.weapon.level += 1;
            Some(format!("{} upgraded to Lv{}!", player.weapon.name(), player.weapon.level))
        }
        ShopItem::BuyBook(kind) => {
            let book = Book::new(kind.clone());
            let name = book.name().to_string();
            player.books.push(book);
            Some(format!("Acquired {}!", name))
        }
        ShopItem::BuyPotion(potion) => {
            let name = potion.name().to_string();
            player.potions.push(potion.clone());
            Some(format!("Bought {}!", name))
        }
        ShopItem::UpgradeHp => {
            player.max_hp += 10;
            player.hp += 10;
            player.upgrades.hp_level += 1;
            Some(format!("Max HP is now {}!", player.max_hp))
        }
        ShopItem::UpgradeAttack => {
            player.base_attack += 2;
            player.upgrades.attack_level += 1;
            Some(format!("Attack is now {}!", player.base_attack))
        }
        ShopItem::UpgradeArmor => {
            player.armor += 1;
            player.upgrades.armor_level += 1;
            Some(format!("Armor is now {}!", player.armor))
        }
        ShopItem::UpgradeDodge => {
            player.dodge_chance += 0.02;
            player.upgrades.dodge_level += 1;
            Some(format!("Dodge is now {}%!", (player.dodge_chance * 100.0) as i32))
        }
        ShopItem::UpgradePotionCapacity => {
            player.max_potions += 1;
            player.upgrades.potion_capacity_level += 1;
            Some(format!("Can now carry {} potions!", player.max_potions))
        }
    }
}

/// Generate shop inventory based on stage
pub fn generate_shop_items(stage: u32, player: &Player) -> Vec<ShopItem> {
    let mut items = Vec::new();

    // Always offer potions
    items.push(ShopItem::BuyPotion(Potion::Healing { power: 30 + stage as i32 }));
    if stage >= 3 {
        items.push(ShopItem::BuyPotion(Potion::Purity));
    }
    if stage >= 5 {
        items.push(ShopItem::BuyPotion(Potion::Lifesteal));
    }

    // Weapon upgrade always available
    if player.weapon.level < 10 {
        items.push(ShopItem::UpgradeWeapon);
    }

    // Offer new weapons periodically
    if stage >= 5 {
        items.push(ShopItem::BuyWeapon(WeaponKind::VenomScythe));
    }
    if stage >= 8 {
        items.push(ShopItem::BuyWeapon(WeaponKind::CrystalScythe));
    }
    if stage >= 12 {
        items.push(ShopItem::BuyWeapon(WeaponKind::ThunderScythe));
    }
    if stage >= 15 {
        items.push(ShopItem::BuyWeapon(WeaponKind::VampireScythe));
    }
    if stage >= 20 {
        items.push(ShopItem::BuyWeapon(WeaponKind::GiantScythe));
    }

    // Books
    if player.books.len() < 3 {
        if stage >= 3 {
            items.push(ShopItem::BuyBook(BookKind::Strength));
        }
        if stage >= 6 {
            items.push(ShopItem::BuyBook(BookKind::Health));
        }
        if stage >= 10 {
            items.push(ShopItem::BuyBook(BookKind::Wisdom));
        }
        if stage >= 12 {
            items.push(ShopItem::BuyBook(BookKind::Wealth));
        }
        if stage >= 15 {
            items.push(ShopItem::BuyBook(BookKind::Speed));
        }
        if stage >= 18 {
            items.push(ShopItem::BuyBook(BookKind::Purity));
        }
    }

    // Stat upgrades
    items.push(ShopItem::UpgradeHp);
    items.push(ShopItem::UpgradeAttack);
    items.push(ShopItem::UpgradeArmor);
    if player.upgrades.dodge_level < 5 {
        items.push(ShopItem::UpgradeDodge);
    }
    if player.upgrades.potion_capacity_level < 3 {
        items.push(ShopItem::UpgradePotionCapacity);
    }

    items
}
