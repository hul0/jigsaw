use rand::seq::SliceRandom;
use rand::Rng;

pub fn generate_memorable_password() -> String {
    let adjs = [
        "Happy", "Sunny", "Fast", "Clever", "Brave", "Calm", "Eager", "Fair", "Gentle", "Jolly", "Kind", "Lively", "Nice", "Proud", "Silly", "Witty", "Zest", "Bold", "Cool", "Deep"
    ];
    let nouns = [
        "Panda", "Tiger", "Eagle", "Lion", "Bear", "Wolf", "Fox", "Hawk", "Owl", "Deer", "Cat", "Dog", "Fish", "Bird", "Frog", "Toad", "Shark", "Whale", "Seal", "Crab"
    ];
    let verbs = [
        "Run", "Jump", "Swim", "Fly", "Walk", "Sing", "Dance", "Read", "Write", "Draw", "Cook", "Eat", "Sleep", "Dream", "Wake", "Look", "See", "Hear", "Touch", "Feel"
    ];
    
    let mut rng = rand::thread_rng();
    
    let adj = adjs.choose(&mut rng).unwrap();
    let noun = nouns.choose(&mut rng).unwrap();
    let verb = verbs.choose(&mut rng).unwrap();
    let num = rng.gen_range(10..99);
    let special = ["!", "@", "#", "$", "%", "&"];
    let sym = special.choose(&mut rng).unwrap();

    format!("{}{}{}{}{}", adj, noun, verb, num, sym)
}
