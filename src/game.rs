#![feature(mpmc_channel)]
use std::{
    convert::{ TryFrom, From },
    collections::{ HashSet, HashMap },
    ops::Index,
    thread,
    time::Duration,
    sync::{
        mpmc::{ Receiver, Sender },
    },
};
use rand::{thread_rng, Rng};
use crate::setup;
type CardPile = Vec<(Card, Vec<(Suit, Rank)>)>;
impl Player<'_> {
    pub fn new<'a>(character: Character, role: Role, config: Option<&Config>, extra_attrs: Option<Vec<Attribute>>) -> Player {
        let mut attributes: HashSet<Attribute> = HashSet::new();
        let health = match &character {
            Character::ElGringo | Character::PaulRegret => {
                attributes.insert(Attribute::LoweredMaxHealth); 
                3
            },
            _ => 4
        };
        let mut player = Player {
            health: health,
            weapon: Weapon::Colt45,
            attributes: attributes,
            attributes_num: HashMap::new(),
            character: character,
            role: role,
            lower_hand: Vec::new(),
            upper_hand: Vec::new(),
            config: config,
        };
        if let Some(vec) = extra_attrs {
            for x in vec {
                let attr_num_res = x.try_num(config.clone());
                match attr_num_res {
                    Some(attr_num) => { player.add_attr_num(attr_num); }
                    None => { player.attributes.insert(x); }
                }
            };
        }
        player
    }
    pub fn health(&self) -> usize {
        self.health
    }
    pub fn weapon(&self) -> &Weapon {
        &self.weapon
    }
    pub fn attr(&self) -> &HashSet<Attribute> {
        &self.attributes
    }
    pub fn attr_num(&self) -> &HashMap<AttrNum, usize> {
        &self.attributes_num
    }
    /*fn mod_attr_ref<T, A, N>(&mut self, attr: &Attribute, fn_attr: A, fn_attr_num: N) -> T
        where
            A: Fn(&mut Self, &Attribute) -> T,
            N: Fn(&mut Self, &AttrNum) -> T
    {
        let attr_num_res = attr.try_num(self.config.clone());
        match attr_num_res {
            Some(attr_num) => fn_attr_num(self, &attr_num),
            None => fn_attr(self, attr)
        }
    }
    fn mod_attr<T, A, N>(&mut self, attr: Attribute, fn_attr: A, fn_attr_num: N) -> T
        where
            A: Fn(&mut Self, Attribute) -> T,
            N: Fn(&mut Self, AttrNum) -> T
    {
        let attr_num_res = attr.try_num(self.config.clone());
        match attr_num_res {
            Some(attr_num) => fn_attr_num(self, attr_num),
            None => fn_attr(self, attr)
        }
    }*/
    pub fn has_attr(&self, attr: &Attribute) -> bool {
        let attr_num_res = attr.try_num(self.config.clone());
        match attr_num_res {
            Some(attr_num) => self.has_attr_num(&attr_num),
            None => self.attr().contains(attr)
        }
    }
    pub fn has_attr_num(&self, attr: &AttrNum) -> bool {
        self.attributes_num.contains_key(attr)
    }
    pub fn add_attr(&mut self, attr: Attribute) {
        let attr_num_res = attr.try_num(self.config.clone());
        match attr_num_res {
            Some(attr_num) => { self.add_attr_num(attr_num); }
            None => { self.attributes.insert(attr); }
        }
    }
    pub fn add_attr_num(&mut self, attr: AttrNum) {
        let mut attrs = &mut self.attributes_num;
        if let Some(val) = attrs.get_mut(&attr) {
            *val += 1;
        } else {
            attrs.insert(attr, 1); 
        }
    }
    pub fn rm_attr(&mut self, attr: &Attribute) -> bool {
        let from_res = attr.try_num(self.config.clone());
        match from_res {
            Some(attr_num) => self.rm_attr_num(&attr_num).is_some(),
            None => self.attributes.remove(attr)
        }
    }
    pub fn rm_attr_num(&mut self, attr: &AttrNum) -> Option<usize> {
        let attrs = &mut self.attributes_num;
        let mut count = attrs.get_mut(&attr)?;
        if *count > 0 {
            *count -= 1; 
        }
        if *count == 0 {
            attrs.remove(&attr)?;
            return Some(0);
        }
        Some(*count)
    }
    pub fn pull_card(&mut self, deck: &mut Deck, default: &CardPile) -> (Card, Suit, Rank) {
        let tuple = deck.rm_card(default);
        dbg!(tuple.0, tuple.1, tuple.2);
        self.upper_hand.push(tuple.0);
        tuple
    }
    pub fn rm_upper(&mut self, index: usize) -> Option<Card> {
        let hand = self.upper_hand();
        match hand.len() {
            0 => None,
            _ => Some(hand.remove(index)),
        }
    }
    pub fn add_upper(&mut self, card: Card) {
        let hand = self.upper_hand();
        hand.push(card);
    } 
    pub fn rm_lower(&mut self, index: usize) -> Option<LowerCard> {
        let hand = self.lower_hand();
        match hand.len() {
            0 => None,
            _ => Some(hand.remove(index)),
        }
    }
    pub fn add_lower(&mut self, card: LowerCard) {
        let hand = self.lower_hand();
        hand.push(card);
    } 
    pub fn steal_lower(&mut self, target: &mut Player) -> bool {
        let hand = target.lower_hand();
        let rng_res = rng(0..hand.len());
        if let None = rng_res {
            return false;
        }
        if hand.len() == 0 {
            return false;
        }
        let card = hand.remove(rng_res.unwrap());
        self.add_lower(card.try_into().unwrap());
        true
    }
    pub fn steal_upper(&mut self, target: &mut Player) -> bool {
        let hand = target.upper_hand();
        let rng_res = rng(0..hand.len());
        if let None = rng_res {
            return false;
        }
        if hand.len() == 0 {
            return false;
        }
        let card = hand.remove(rng_res.unwrap());
        self.add_upper(card);
        true
    }
    pub fn character(&self) -> &Character {
        &self.character
    }
    pub fn role(&self) -> &Role {
        &self.role
    }
    pub fn upper_hand(&mut self) -> &mut Vec<Card> {
        &mut self.upper_hand
    }
    pub fn lower_hand(&mut self) -> &mut Vec<LowerCard> {
        &mut self.lower_hand
    }
    pub fn config(&self) -> Option<&Config> {
        self.config.clone()
    }
}
impl Default for Player<'_> {
    fn default() -> Self {
        use Attribute::*;
        Player {
            health: 4,
            weapon: Weapon::Colt45,
            attributes: HashSet::new(),
            attributes_num: HashMap::new(),
            character: Character::CalamityJanet,
            role: Role::Outlaw,
            lower_hand: Vec::new(),
            upper_hand: Vec::new(),
            config: None,
        }
    }
}
impl Attribute {
    pub fn try_num(&self, config: Option<&Config>) -> Option<AttrNum> {
        if let None = &config {
            return AttrNum::try_from(self).ok();
        }
        if let None = &config?.num_attrs() {
            return AttrNum::try_from(self).ok();
        }
        let allowed_attrs = config?.num_attrs()?;
        let attr = allowed_attrs
            .iter()
            .find(|x| x == &self)?;
        AttrNum::try_from(attr).ok()
    }
}
pub struct Deck {
    pub inner: CardPile,
}
impl Deck {
    pub fn new(inner: CardPile) -> Self {
        Deck { inner: inner }
    }
    pub fn rm_card(&mut self, default: &CardPile) -> (Card, Suit, Rank) {
        let len = self.len();
        let rng1 = rng(0..len).expect("Vec should not be empty");
        let selection = self.inner.get_mut(rng1).unwrap();
        let card = selection.0.clone();
        let mut vec = &mut selection.1;
        let rng2 = rng(0..vec.len()).expect("Vec should not be empty");
        let (suit, rank) = vec.swap_remove(rng2); 
        if vec.len() > 0 {
            return (card, suit, rank);
        }
        self.inner.swap_remove(rng1);
        println!("last one: {:?}", self.inner());
        if self.inner().len() == 0 {
            self.inner = default.to_vec();
            println!("restocked: {:?}", self.inner());
        }
        (card, suit, rank)
    }
    pub fn inner(&self) -> &CardPile {
        &self.inner
    }
    pub fn inner_mut(&mut self) -> &mut CardPile {
        &mut self.inner
    }
    pub fn extract_inner(self) -> CardPile {
        self.inner
    }
    pub fn restock(&mut self, default: CardPile) -> bool {
        true
    }
    pub fn len(&self) -> usize {
        self.inner.len()
    }
}
#[repr(u8)]
pub enum Weapon {
    Colt45,
    Volcanic,
    Schofield,
    Remington,
    Carabine,
    Winchester
}
#[repr(u8)]
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
#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum Card {
    Bang,
    Miss,
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
    Beer,
    CatBalou,
    Duel,
    Gatling,
    Store,
    Indians,
    Panic,
    Saloon,
    Stagecoach,
    WellsFargo,
}
#[repr(u8)]
pub enum LowerCard {
    Jail,
    Mustang,
    Dynamite,
    Scope, 
}
#[repr(u8)]
pub enum Role {
    Sheriff,
    Deputy,
    Outlaw,
    Renegade
}
#[repr(u8)]
#[derive(Eq, Hash, PartialEq, Debug)]
pub enum Attribute {
    Barrel,
    Jailed,
    Dynamite, 
    ExtraRange,
    ExtraDistance,
    LoweredMaxHealth,
    BangMissSwap,
    HitStealCard,
    HitPullCard,
    PullStealCard,
    Pull3Choose2,
    Draw2Choose,
    PullFromDiscard,
    Discard2Heal,
    DoubleMissed,
    RunOutPull,
    KillStealCards,
    NoBangLimit,
    SecondPullGamble,
}
#[repr(u8)]
#[derive(Eq, Hash, PartialEq, Debug)]
pub enum AttrNum {
    ExtraRange,
    ExtraDistance,
    Barrel,
}
#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum Suit {
    Heart,
    Diamond,
    Spade,
    Club
}
#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum Rank {
    One = 1,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
    Ace
}
#[derive(Clone, Copy)]
pub enum PlayerAction {
    PlayCard(usize),
    ActivateCard(usize),
    Choose(usize),
    PullCard,
}
#[derive(Clone, Copy)]
pub enum GameAction {
    Choose,

}
pub struct Player<'a> {
    health: usize,
    weapon: Weapon,
    attributes: HashSet<Attribute>,
    attributes_num: HashMap<AttrNum, usize>,
    character: Character,
    role: Role,
    lower_hand: Vec<LowerCard>,
    upper_hand: Vec<Card>,
    config: Option<&'a Config>,
}  
pub struct Config {
    num_attrs: Option<HashSet<Attribute>>
}
impl Config {
    pub fn new(num_attrs: Option<HashSet<Attribute>>) -> Self {
        Config {
            num_attrs: num_attrs,
        }
    }
    pub fn num_attrs(&self) -> Option<&HashSet<Attribute>> {
        self.num_attrs.as_ref()
    }
}
impl Default for Config {
    fn default() -> Self {
        Config {
            num_attrs: None
        }
    }
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
            BlackJack => Self::SecondPullGamble,
            JesseJones => Self::PullStealCard,
            Jourdannais => Self::Barrel,
            KitCarlson => Self::HitStealCard,
            LuckyDuke => Self::Draw2Choose,
            PaulRegret => Self::ExtraDistance,
            PedroRamirez => Self::PullFromDiscard,
            RoseDoolan => Self::ExtraRange,
            SidKetchum => Self::Discard2Heal,
            SlabTheKiller => Self::DoubleMissed,
            SuzyLafayette => Self::RunOutPull,
            VultureSam => Self::KillStealCards,
            WillyTheKid => Self::NoBangLimit,
        }
    }
}
impl<'a> TryFrom<&'a Attribute> for AttrNum {
    type Error = &'a Attribute;
    fn try_from(value: &'a Attribute) -> Result<Self, Self::Error> {
        use Attribute::*;
        match value {
            Barrel => Ok(Self::Barrel),
            ExtraRange => Ok(Self::ExtraRange),
            ExtraDistance => Ok(Self::ExtraDistance),
            val => Err(val)
        }
    }
}
impl From<&AttrNum> for Attribute {
    fn from(value: &AttrNum) -> Self {
        use AttrNum::*;
        match value {
            Barrel => Self::Barrel,
            ExtraRange => Self::ExtraRange,
            ExtraDistance => Self::ExtraDistance,
        }
    }
}
impl From<usize> for Card {
    fn from(value: usize) -> Card {
        use Card::*;
        match value {
            0 => Bang,
            1 => Miss,
            2 => Barrel,
            3 => Dynamite,
            4 => Jail,
            5 => Mustang,
            6 => Remington,
            7 => Carabine,
            8 => Schofield,
            9 => Scope,
            10 => Volcanic,
            11 => Winchester,
            12 => Beer,
            13 => CatBalou,
            14 => Duel,
            15 => Gatling,
            16 => Store,
            17 => Indians,
            18 => Panic,
            19 => Saloon,
            20 => Stagecoach,
            21 => WellsFargo,
            _ => unreachable!()
        }
    }
}
fn rng(range: std::ops::Range<usize>) -> Option<usize> {
    if range.is_empty() { return None; }
    let index: usize = thread_rng().gen_range(range);
    Some(index)
}
pub fn start_game(mut tx: Sender<GameAction>, rx: Receiver<PlayerAction>) { while let Ok(action) = rx.recv() {
    println!("action");
}}

#[cfg(test)]
mod tests {
    use crate::{Player, Character::*, Role::*, Deck, Attribute::*, Config};
    use std::collections::HashSet;

    #[test]
    fn pull_card() {
        let mut player = Player::default();  
        assert_eq!(player.upper_hand().len(), 0);
        let mut deck = Deck::default();
        let default = Deck::default().extract_inner();
        player.pull_card(&mut deck, &default);
        assert_ne!(player.upper_hand().len(), 0);
    }

    #[test]
    fn attributes() {
        let config = Config::new(Some(HashSet::from([ExtraRange, Barrel])));
        let player = Player::new(CalamityJanet, Sheriff, Some(&config), None);
        attributes_test(player)
    }

    //#[test]
    //fn attributes_default() {
    //    let player = Player::default();
    //    attributes_test(player)
    //}

    fn attributes_test(mut player: Player) {
        dbg!(player.attr_num());
        dbg!(player.attr());
        assert!(player.attr().len() == 0);
        assert!(player.attr_num().len() == 0);

        player.add_attr(Jailed);
        dbg!(player.attr_num());
        dbg!(player.attr());
        assert!(player.attr().len() == 1);
        assert!(player.attr_num().len() == 0);

        player.add_attr(ExtraRange);
        dbg!(player.attr_num());
        dbg!(player.attr());
        assert!(player.attr().len() == 1);
        assert!(player.attr_num().len() == 1);

        player.add_attr(ExtraRange);
        dbg!(player.attr_num());
        dbg!(player.attr());
        assert!(player.attr().len() == 1);
        assert!(player.attr_num().len() == 1);

        player.add_attr(ExtraDistance);
        dbg!(player.attr_num());
        dbg!(player.attr());
        assert!(player.attr().len() == 2);
        assert!(player.attr_num().len() == 1);
        {
            let res = player.rm_attr(&Dynamite);
            assert!(player.attr().len() == 2);
            assert!(player.attr_num().len() == 1);
            assert!(res == false);
        }{ 
            let res = player.rm_attr(&Jailed);
            assert!(player.attr().len() == 1);
            assert!(player.attr_num().len() == 1);
            assert!(res == true);
        }{ 
            dbg!(player.attr_num());
            let res = player.rm_attr(&ExtraRange);
            dbg!(player.attr_num());
            assert!(player.attr().len() == 1);
            assert!(player.attr_num().len() == 1);
            assert!(res == true);
        }{ 
            let res = player.has_attr(&ExtraRange);
            assert!(res == true);
        }{
            let res = player.rm_attr(&ExtraRange);
            dbg!(player.attr_num());
            assert!(player.attr().len() == 1);
            assert!(player.attr_num().len() == 0);
            assert!(res == true);
        }{
            let res = player.has_attr(&ExtraRange);
            assert!(res == false);
        }{
            let res = player.has_attr(&Dynamite);
            assert!(res == false);
        }{
            let res = player.has_attr(&ExtraDistance);
            assert!(res == true);
        }
    }

    #[test]
    fn steal() {
        let mut player1 = Player::default();  
        let mut player2 = Player::default();  
        let mut deck = Deck::default();
        let default = Deck::default().extract_inner();
        player1.pull_card(&mut deck, &default);
        assert!(
            player1.upper_hand().len() > 0 &&
            player2.upper_hand().len() == 0
        );
        player2.steal_upper(&mut player1);
        assert!(
            player1.upper_hand().len() == 0 &&
            player2.upper_hand().len() > 0
        );
    }
}
