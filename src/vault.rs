/* vault

        * 8 -  1 (Vault: 30)

        4 * 11 *

        + 4 -  18

 Orb 22 O - 9  *

 22

*/

use pathfinding::prelude::*;
use crate::Op;

#[derive(Copy,Clone,Debug,Hash,PartialEq,Eq)]
pub enum OrbColour {
    GreenForAdd,
    RedForSub,
    OrangeForMult,
}

#[derive(Copy,Clone,Debug,Hash,PartialEq,Eq)]
pub struct OrbState {
    value: i32,
    colour: Option<OrbColour>,
    position: (u8,u8),
}
/*
 *   8 -  1 (Vault: 30)
 4   * 11 *
 +   4 -  18
 =22 - 9  *
 */
pub fn get_room_value(p: (u8,u8)) -> Option<i32> {
    match p {
        (0,0) => Some(22),
        (0,2) => Some(9),
        (1,1) => Some(4),
        (1,3) => Some(18),
        (2,0) => Some(4),
        (2,2) => Some(11),
        (3,1) => Some(8),
        (3,3) => Some(1),
        _ => None
    }
}
pub fn get_room_orb_colour(p: (u8,u8)) -> Option<OrbColour> {
    match p {
        (0,1) | (1,2) | (3,2) => Some(OrbColour::RedForSub),
        (0,3) | (2,1) | (2,3) | (3,0) => Some(OrbColour::OrangeForMult),
        (1,0) => Some(OrbColour::GreenForAdd),
        _ => None
    }
}
pub fn do_step(s: &OrbState, p: (u8,u8)) -> OrbState {
    let val = get_room_value(p);
    let mut ans = s.clone();
    ans.position = p;
    //println!("step to {:?} from {:?} = ",p,s);
    ans.colour = get_room_orb_colour(p);
    match s.colour {
        None => (),
        Some(OrbColour::GreenForAdd) => {
            ans.value += val.expect("Room should be valued")
        },
        Some(OrbColour::OrangeForMult) => ans.value *= val.expect("Room should be valued"),
        Some(OrbColour::RedForSub) => ans.value -= val.expect("Room should be valued"),
    }
    //println!("      {:?}",ans);
    ans
}
pub fn neighbours(s: &OrbState) -> Vec<OrbState> {
    let (y,x) = s.position;
    let all_pos = vec![(y+1,x),(y-1,x),(y,x-1),(y,x+1)];
    all_pos.into_iter()
        .filter(|&p| p.0 >= 0 && p.1 >= 0 && p.0 < 4 && p.1 < 4 && p != (0,0))
        .map(|p| do_step(s,p))
        .filter(|s| s.position != (3,3) || s.value == 30)
        .collect()
}
pub fn goal(s: &OrbState) -> bool {
    s.position == (3,3) && s.value == 30
}
pub fn find_sol() {
    let foo= pathfinding::directed::bfs::bfs(&OrbState{
        position: (0,0),
        value: 22,
        colour: None
    },neighbours, goal);
    println!("{:?}",foo);
}

/*
NEENNSWNEE
Some([
OrbState { value: 22, colour: None, position: (0, 0) },
OrbState { value: 22, colour: Some(GreenForAdd), position: (1, 0) },
OrbState { value: 26, colour: None, position: (1, 1) },
OrbState { value: 26, colour: Some(RedForSub), position: (1, 2) },
OrbState { value: 15, colour: None, position: (2, 2) },
OrbState { value: 15, colour: Some(RedForSub), position: (3, 2) },
OrbState { value: 4, colour: None, position: (2, 2) },
OrbState { value: 4, colour: Some(OrangeForMult), position: (2, 1) },
OrbState { value: 32, colour: None, position: (3, 1) },
OrbState { value: 32, colour: Some(RedForSub), position: (3, 2) },
OrbState { value: 31, colour: None, position: (3, 3) },
OrbState { value: 31, colour: Some(RedForSub), position: (3, 2) },
OrbState { value: 30, colour: None, position: (3, 3) }])

NEENWSEEWNNE
Some([
OrbState { value: 22, colour: None, position: (0, 0) },
OrbState { value: 22, colour: Some(GreenForAdd), position: (1, 0) },
OrbState { value: 26, colour: None, position: (1, 1) },
OrbState { value: 26, colour: Some(RedForSub), position: (1, 2) },
OrbState { value: 15, colour: None, position: (2, 2) },
OrbState { value: 15, colour: Some(OrangeForMult), position: (2, 1) },
OrbState { value: 60, colour: None, position: (1, 1) },
OrbState { value: 60, colour: Some(RedForSub), position: (1, 2) },
OrbState { value: 42, colour: None, position: (1, 3) },
OrbState { value: 42, colour: Some(RedForSub), position: (1, 2) },
OrbState { value: 31, colour: None, position: (2, 2) },
OrbState { value: 31, colour: Some(RedForSub), position: (3, 2) },
OrbState { value: 30, colour: None, position: (3, 3) }])


*/


//TOUopp8OMbdp
//qbdMO8qqoUOT