pub mod game {
    use std::{
        convert::{ TryFrom, From },
        collections::{ HashSet, HashMap }
    };
    use rand::{thread_rng, Rng};
    pub enum Weapon {
        Colt45,
        Volcanic,
        Schofield,
        Remington,
        Carabine,
        Winchester
    }
    pub enum Character {
        CalamityJanet,
        ElGringo,
        BartCassidy,
        BlackJack,
        JesseJones,
        Jourdannais,
        KitCarlson,
        LuckyDuke,
        PaulRegret,
        PedroRamirez,
        RoseDoolan,
        SidKetchum,
        SlabTheKiller,
        SuzyLafayette,
        VultureSam,
        WillyTheKid
    }
    #[derive(Hash, Eq, PartialEq)]
    pub enum Card {
        Barrel,
        Dynamite,
        Jail,
        Mustang,
        Remington,
        Carabine,
        Schofield,
        Scope,
        Volcanic,
        Winchester,
        Bang,
        Beer,
        CatBalou,
        Duel,
        Gatling,
        Store,
        Indians,
        Miss,
        Panic,
        Saloon,
        Stagecoach,
        WellsFargo,
    }
    pub enum LowerCard {
        Jail,
        Mustang,
        Dynamite,
        Scope,
    }
    pub enum Role {
        Sheriff,
        Deputy,
        Outlaw,
        Renegade
    }
    #[derive(Hash, Eq, PartialEq)]
    pub enum Attribute {
        Barrel,
        ExtraRange,
        ExtraDistance,
        Jailed,
        Dynamite, 
        LoweredHealth,
        BangMissSwap,
        HitStealCard,
        HitPullCard,
        PullStealCard,
        Pull3Choose2,
        Draw2Choose,
        PullFromDiscard,
        Discard2Heal,
        DoubleMissed,
        NoCardPull,
        KillStealCards,
        NoBangLimit,
        SecondPullShowGamble,
        Scope,
    }
    pub struct Player {
        health: u8,
        weapon: Weapon,
        attributes: HashSet<Attribute>,
        attributes_num: HashMap<Attribute, u8>,
        character: Character,
        role: Role,
        lower_hand: Vec<LowerCard>,
        upper_hand: Vec<Card>,
    }  
    impl TryFrom<Card> for LowerCard {
        type Error = ();
        fn try_from(value: Card) -> Result<Self, Self::Error> {
            use Card::*;
            match value {
                Jail => Ok(Self::Jail),
                Mustang => Ok(Self::Mustang),
                Dynamite => Ok(Self::Dynamite),
                Scope => Ok(Self::Scope),
                _ => Err(())
            }
        }
    }
    impl From<LowerCard> for Card {
        fn from(value: LowerCard) -> Self {
            use LowerCard::*;
            match value {
                Jail => Self::Jail,
                Mustang => Self::Mustang,
                Dynamite => Self::Dynamite,
                Scope => Self::Scope,
            }
        }
    }
    impl TryFrom<Card> for Attribute {
        type Error = ();
        fn try_from(value: Card) -> Result<Self, Self::Error> {
            use Card::*;
            match value {
                Jail => Ok(Self::Jailed),
                Barrel => Ok(Self::Barrel),
                Mustang => Ok(Self::ExtraRange),
                Scope => Ok(Self::ExtraDistance),
                Dynamite => Ok(Self::Dynamite),
                _ => Err(())
            }
        }
    }
    impl From<&Character> for Attribute {
        fn from(value: &Character) -> Self {
            use Character::*;
            match value {
                CalamityJanet => Self::BangMissSwap,
                ElGringo => Self::HitStealCard,
                BartCassidy => Self::HitPullCard,
                BlackJack => Self::SecondPullShowGamble,
                JesseJones => Self::PullStealCard,
                Jourdannais => Self::Barrel,
                KitCarlson => Self::HitStealCard,
                LuckyDuke => Self::Draw2Choose,
                PaulRegret => Self::ExtraDistance,
                PedroRamirez => Self::PullFromDiscard,
                RoseDoolan => Self::Scope,
                SidKetchum => Self::Discard2Heal,
                SlabTheKiller => Self::DoubleMissed,
                SuzyLafayette => Self::NoCardPull,
                VultureSam => Self::KillStealCards,
                WillyTheKid => Self::NoBangLimit,
            }
        }
    }
    impl Player {
        pub fn new(character: Character, role: Role, extra_attributes: Option<HashSet<Attribute>>) -> Player {
            let mut attributes = if let Some(hash_set) = extra_attributes {
                hash_set
            } else {
                HashSet::<Attribute>::new()
            };
            let health: u8 = match &character {
                Character::ElGringo | Character::PaulRegret => {
                    attributes.insert(Attribute::LoweredHealth); 
                    3
                },
                _ => 4
            };
            let player = Player {
                health: health,
                weapon: Weapon::Colt45,
                attributes: attributes,
                attributes_num: HashMap::new(),
                character: character,
                role: role,
                lower_hand: Vec::new(),
                upper_hand: Vec::new()
            };
            player
        }
        pub fn health(&self) -> u8 {
            self.health
        }
        pub fn weapon(&self) -> &Weapon {
            &self.weapon
        }
        pub fn attr(&self) -> &HashSet<Attribute> {
            &self.attributes
        }
        pub fn attr_num(&self) -> &HashMap<Attribute, u8> {
            &self.attributes_num
        }
        pub fn add_attr(&mut self, attr: Attribute) {
            self.attributes.insert(attr); 
        }
        pub fn rm_attr(&mut self, attr: &Attribute) {
            self.attributes.remove(attr); 
        }
        pub fn has_attr(&self, attr: &Attribute) -> bool {
            self.attributes.contains(attr)
        }
        pub fn add_attr_num(&mut self, attr: Attribute, amount: u8) {
            self.attributes_num.insert(attr, amount); 
        }
        pub fn rm_attr_num(&mut self, attr: &Attribute) {
            self.attributes_num.remove(attr); 
        }
        pub fn pull_card(&mut self, deck: &mut HashMap<Card, u8>) -> bool {
            let mut rand = thread_rng();
            let len = deck.len().try_into().expect("Pull card math ungabunga");
            let index: u8 = rand.gen_range(0..len);
            loop {
                let variant = Card[index];
                let card = deck.remove(variant);
            }
            //enum indexing
            println!("{}", index);
            true
        }
        pub fn steal_card(&mut self, target: &mut Player, index: u8) -> bool {
            todo!()
        }
        pub fn has_attr_num(&self, attr: &Attribute) -> bool {
            self.attributes_num.contains_key(attr)
        }
        pub fn character(&self) -> &Character {
            &self.character
        }
        pub fn role(&self) -> &Role {
            &self.role
        }
        pub fn upper_hand(&self) -> &Vec<Card> {
            &self.upper_hand
        }
        pub fn lower_hand(&self) -> &Vec<LowerCard> {
            &self.lower_hand
        }
    }
}
