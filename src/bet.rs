
use std::fmt::Debug;

use anyhow::{Result, bail};

use crate::{Context, private_reply};

#[derive(poise::ChoiceParameter, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Dice {
    #[name = "1"] _1,
    #[name = "2"] _2,
    #[name = "3"] _3,
    #[name = "4"] _4,
    #[name = "5"] _5,
    #[name = "6"] _6,
}

#[derive(poise::ChoiceParameter, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Total {
    #[name = "4"] _4,
    #[name = "5"] _5,
    #[name = "6"] _6,
    #[name = "7"] _7,
    #[name = "8"] _8,
    #[name = "9"] _9,
    #[name = "10"] _10,
    #[name = "11"] _11,
    #[name = "12"] _12,
    #[name = "13"] _13,
    #[name = "14"] _14,
    #[name = "15"] _15,
    #[name = "16"] _16,
    #[name = "17"] _17,
}

#[derive(poise::ChoiceParameter)]
pub enum Four {
    #[name = "6543"] _6543,
    #[name = "6532"] _6532,
    #[name = "5432"] _5432,
    #[name = "4321"] _4321,
}

#[allow(non_camel_case_types)]
#[derive(Debug)]
pub enum Bets {
    small,
    big,
    odd,
    even,
    any(Dice),
    any2(Dice, Dice),
    any3(Dice, Dice, Dice),
    any21(Dice, Dice),
    double(Dice),
    triple(Dice),
    all,
    total(Total),
    four(Four),
}

#[derive(Clone, Copy)]
pub struct Dices(Dice, Dice, Dice);

impl Dices {
    pub fn new(d1: Dice, d2: Dice, d3: Dice) -> Self { Self(d1, d2, d3) }
    
    pub fn sum(self) -> u8 { self.0 as u8 + self.1 as u8 + self.2 as u8 + 3 }
    
    pub fn total(self) -> Option<Total> {
        let i =self.sum();
        if (4..=17).contains(&i) { Some(unsafe { std::mem::transmute(i - 4) }) }
        else { None }
    }
    
    pub fn odd(self) -> bool { self.sum() % 2 == 1 }
    pub fn even(self) -> bool { !self.odd() }
    
    pub fn triple(self) -> Option<Dice> { if self.0 == self.1 && self.1 == self.2 { Some(self.0) } else { None } }
    pub fn double(self) -> Option<Dice> {
        if self.0 == self.1 || self.0 == self.2 { Some(self.0) }
        else if self.1 == self.2 { Some(self.1) }
        else { None }
    }
    
    pub fn any(self, d: Dice) -> bool { self.0 == d || self.1 == d || self.2 == d }
}

impl Bets {
    pub fn play(self, d: Dices) -> u64 {
        use Bets::*;
        use Total::*;
        
        match self {
            small | big | odd | even if d.triple().is_some() => 0,
            small if (4..=10).contains(&d.sum()) => 1,
            big if (11..=17).contains(&d.sum()) => 1,
            odd if d.odd() => 1,
            even if d.even() => 1,
            any(dice) if d.triple() == Some(dice) => 12,
            any(dice) if d.double() == Some(dice) => 2,
            any(dice) if d.any(dice) => 1,
            any2(d1, d2) if d.any(d1) && d.any(d2) => 6,
            any3(d1, d2, d3) if d.any(d1) && d.any(d2) && d.any(d3) => 33,
            any21(d1, d2) if d.double() == Some(d1) && d.any(d2) => 60,
            double(dice) if d.double() == Some(dice) => 11,
            triple(dice) if d.triple() == Some(dice) => 190,
            all if d.triple().is_some() => 33,
            total(n) if d.total() == Some(n) => match n {
                _4 | _17 => 65,
                _5 | _16 => 33,
                _6 | _15 => 19,
                _7 | _14 => 12,
                _8 | _13 => 8,
                _9 | _12 => 7,
                _10 | _11 => 6,
            },
            four(f) => {
                use Dice::*;
                
                let f = match f {
                    Four::_6543 => [_6, _5, _4, _3],
                    Four::_6532 => [_6, _5, _3, _2],
                    Four::_5432 => [_5, _4, _3, _2],
                    Four::_4321 => [_4, _3, _2, _1],
                };
                
                if f.into_iter().filter(|&dice| d.any(dice)).count() == 3 { 60 } else { 0 }
            }
            _ => 0,
        }
    }
}

impl Debug for Dice { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{}", *self as u8 + 1) } }
impl Debug for Total { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{}", *self as u8 + 4) } }
impl Debug for Four {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match *self {
            Four::_6543 => "6543",
            Four::_6532 => "6532",
            Four::_5432 => "5432",
            Four::_4321 => "4321",
        })
    }
}

async fn user_bet(context: Context<'_>, amount: u64, bet: Option<Bets>) -> Result<()> {
    if let Some(bet) = bet {
        let f = || unsafe { std::mem::transmute(rand::random_range(1..=6) as u8 - 1) };
        let [d1, d2, d3] = [f(), f(), f()];
        let d = Dices::new(d1, d2, d3);
        
        let mut s = String::new();
        let mut valid = true;
        
        {
            let Ok(mut data) = context.data().write() else { bail!("Lock poisoned") };
            let user = data.get_mut(context.author().id);
            
            if user.money < amount {
                valid = false
            }
            else {
                s = format!("Dice: {:?} + {:?} + {:?} = {}\nBet: {:?}\n", d1, d2, d3, d.sum(), bet);
                
                match bet.play(d) {
                    0 => {
                        s += &format!("You lost :(\nLoss: -{}\n", amount);
                        user.money -= amount;
                    }
                    odds => {
                        let gain = amount * odds;
                        s += &format!("You won!\nGain: +{}\n", gain);
                        user.money += gain;
                    }
                }
                
                s += &format!("Money: {}", user.money);
            }
        }
        
        if !valid { return private_reply(context, "Not enough money!").await }
        
        context.say(s).await?;
        
        Ok(())
    }
    else { private_reply(context, "Must be different number!").await }
}

macro_rules! bet {
    ($doc:meta $b:ident($($a:ident:$t:ty),*) { $e:expr }) => {
        #[$doc]
        #[poise::command(slash_command, prefix_command)]
        pub async fn $b(context: Context<'_>, amount: u64, $($a:$t),*) -> Result<()> {
            let f = || Ok($e);
            user_bet(context, amount, match f() { Ok(bet) => Some(bet), Err(()) => None }).await
        }
    };
}

bet!(doc="win if 4 ≤ total ≤ 10 and not triple (odds: 1)" small() { Bets::small });
bet!(doc="win if 11 ≤ total ≤ 17 and not triple (odds: 1)" big() { Bets::big });
bet!(doc="win if total is odd and not triple (odds: 1)" odd() { Bets::odd });
bet!(doc="win if total is even and not triple (odds: 1)" even() { Bets::even });
bet!(doc="choose 1~6, win if it is thrown (odds: 1~12)" any(d: Dice) { Bets::any(d) });
bet!(doc="choose 2 different 1~6, win if both are thrown (odds: 6)" any2(d1: Dice, d2: Dice) {
    if d1 == d2 { Err(())? } else { Bets::any2(d1, d2) }
});
bet!(doc="choose 3 different 1~6, win if all are thrown (odds: 33)" any3(d1: Dice, d2: Dice, d3: Dice) {
    if d1 == d2 || d1 == d3 { Err(())? } else { Bets::any3(d1, d2, d3) }
});
bet!(doc="choose 2 different 1~6, win if first is thrown double and second is thrown (odds: 60)" any21(d1: Dice, d2: Dice) {
    if d1 == d2 { Err(())? } else { Bets::any21(d1, d2) }
});
bet!(doc="choose 1-6, win if it is double thrown (odds: 11)" double(d: Dice) { Bets::double(d) });
bet!(doc="choose 1-6, win if it is triple thrown (odds: 190)" triple(d: Dice) { Bets::triple(d) });
bet!(doc="win if triple (odds: 33)" all() { Bets::all });
bet!(doc="choose 4-17, win if total is the same (odds: 6~65)" total(t: Total) { Bets::total(t) });
bet!(doc="choose 6543/6532/5432/4321, win if all thrown are different and in selected (odds: 60)" four(f: Four) { Bets::four(f) });

const RULES: &'static str = "
small: win if 4 ≤ total ≤ 10 and not triple (odds: 1)
big: win if 11 ≤ total ≤ 17 and not triple (odds: 1)
odd: win if total is odd and not triple (odds: 1)
even: win if total is even and not triple (odds: 1)
any: choose 1~6, win if it is thrown (odds: 1, 2 if double, 12 if triple)
any2: choose 2 different 1~6, win if both are thrown (odds: 6)
any3: choose 3 different 1~6, win if all are thrown (odds: 33)
any21: choose 2 different 1~6, win if first is thrown double and second is thrown (odds: 60)
double: choose 1~6, win if it is double thrown (odds: 11)
triple: choose 1~6, win if it is triple thrown (odds: 190)
all: win if triple (odds: 33)
total: choose 4~17, win if total is the same (odds: 4/17=65, 5/16=33, 6/15=19, 7/14=12, 8/13=8, 9/12=7, 10/11=6)
four: choose 6543/6532/5432/4321, win if all thrown are different and in selected (odds: 60)
";

#[poise::command(slash_command, prefix_command)]
pub async fn rules(context: Context<'_>) -> Result<()> {
    context.say(RULES).await?;
    Ok(())
}
