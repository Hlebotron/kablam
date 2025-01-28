use std::{
    default::Default,
    ops::Range,
    collections::HashMap,
}; 

use crate::game::{Deck, Card::{self, *}, Rank::{self, *}, Suit::{self, *},};
impl Default for Deck {
    fn default() -> Self {
        let inner: Vec<(Card, Vec<(Suit, Rank)>)> = Vec::from([
            (Bang, vec![
                (Spade, Ace), 
                (Diamond, Two), 
                (Diamond, Three), 
                (Diamond, Four), 
                (Diamond, Five), 
                (Diamond, Six), 
                (Diamond, Seven), 
                (Diamond, Eight), 
                (Diamond, Nine), 
                (Diamond, Ten), 
                (Diamond, Jack), 
                (Diamond, Queen), 
                (Diamond, King), 
                (Diamond, Ace), 
                (Club, Two), 
                (Club, Three), 
                (Club, Four), 
                (Club, Five), 
                (Club, Six), 
                (Club, Seven), 
                (Club, Eight), 
                (Club, Nine), 
                (Heart, Queen), 
                (Heart, King), 
                (Heart, Ace)
            ]),
            
             (Miss, vec![
                (Club, Ten), 
                (Club, Jack), 
                (Club, Queen), 
                (Club, King), 
                (Club, Ace), 
                (Spade, Two), 
                (Spade, Three), 
                (Spade, Four), 
                (Spade, Five), 
                (Spade, Six), 
                (Spade, Seven), 
                (Spade, Eight)
             ]),
            (Barrel, vec![
                (Spade, Queen), 
                (Spade, King)
            ]),
            (Dynamite, vec![
                (Heart, Two)
            ]),
            (Jail, vec![
                (Heart, Four), 
                (Spade, Jack), 
                (Spade, Ten)
            ]),
            (Mustang, vec![
                (Heart, Eight), 
                (Heart, Nine)
            ]),
            (Remington, vec![
                (Club, King)
            ]),
            (Carabine, vec![
                (Club, Ace)
            ]),
            (Schofield, vec![
                (Club, Jack), 
                (Club, Queen), 
                (Spade, King)
            ]),
            (Scope, vec![
             (Spade, Ace)
            ]),
            (Volcanic, vec![
                (Spade, Ten), 
                (Club, Ten)
            ]),
            (Winchester, vec![
                (Spade, Eight)
            ]),
            (Beer, vec![
                (Heart, Six), 
                (Heart, Seven), 
                (Heart, Eight), 
                (Heart, Nine), 
                (Heart, Ten), 
                (Heart, Jack)
            ]),
            (CatBalou, vec![
                (Heart, King), 
                (Diamond, Nine), 
                (Diamond, Ten), 
                (Diamond, Jack)
            ]),
            (Duel, vec![
                (Diamond, Queen), 
                (Spade, Jack), 
                (Club, Eight)
            ]),
            (Gatling, vec![
                (Heart, Ten)
            ]),
            (Store, vec![
                (Club, Nine), 
                (Spade, Queen)
            ]),
            (Indians, vec![
                (Diamond, King), 
                (Diamond, Ace)
            ]),
            (Panic, vec![
                (Heart, Jack), 
                (Heart, Queen), 
                (Heart, Ace), 
                (Diamond, Eight)
            ]),
            (Saloon, vec![
                (Heart, Five)
            ]),
            (Stagecoach, vec![
                (Spade, Nine), 
                (Spade, Nine)
            ]),
            (WellsFargo, vec![
                (Heart, Three)
            ]),
        ]);
        Deck {
            inner: inner
        }  
    }
}
