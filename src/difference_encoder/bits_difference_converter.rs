use jokrey_utilities::general::is_odd;
use crate::rand::Rng;

pub fn get_min_num_bits_encodable(difference: u8) -> u8 {
    //LOGIC: a dif=1 cannot necessarily encode a bit, dif=2 always can(regardless of encoding table permutation and actual bit string).
    //LOGIC: a dif=3 cannot necessarily encode two bits, dif=6 always can(regardless of encoding table permutation and actual bit string).
    if difference == 255 {return 7;}//otherwise difference +1 would overflow
    let encodable = get_num_bits_decodable(difference + 1); //expressively this does not necessarily seem intuitive, but it is nonetheless correct
    if encodable == 0 {
        0
    } else {
        encodable - 1
    }
}
pub fn get_num_bits_decodable(difference: u8) -> u8 {
    //LOGIC: If bits are known to be encoded in the given difference, we now that they could be encoded given their difference.
    //       Therefore they are at the maximum end of encodability
    get_max_num_bits_encodable(difference)
}
pub fn get_max_num_bits_encodable(difference: u8) -> u8 {
    if difference == 0 {
        0
    } else if difference <= 2 {
        1
    } else if difference <= 6 {
        2
    } else if difference <= 14 {
        3
    } else if difference <= 30 {
        4
    } else if difference <= 62 {
        5
    } else if difference <= 126 {
        6
    } else if difference <= 254 {
        7
    } else /*if difference <= 255*/ {
        8
    }
}

#[test]
fn test_min_num_bits_encodable() {
    assert_eq!(0, get_min_num_bits_encodable(0));//max == 0
    assert_eq!(0, get_min_num_bits_encodable(1));//max == 1 (with probability 50%)
    assert_eq!(1, get_min_num_bits_encodable(2));//max == 1
    assert_eq!(1, get_min_num_bits_encodable(3));//max == 2
    assert_eq!(1, get_min_num_bits_encodable(5));//max == 2
    assert_eq!(2, get_min_num_bits_encodable(6));//max == 2
    assert_eq!(2, get_min_num_bits_encodable(7));//max == 3
    assert_eq!(6, get_min_num_bits_encodable(253));//max == 7
    assert_eq!(7, get_min_num_bits_encodable(254));//max == 7
    assert_eq!(7, get_min_num_bits_encodable(255));//max == 8 (though unlikely)
}

pub fn calculate_worst_case_difference_for(num_bits: u8) -> Option<u8> {
    //*2 + 2
    match num_bits {
        0 => Some(0),
        1 => Some(2),
        2 => Some(6),
        3 => Some(14),
        4 => Some(30),
        5 => Some(62),
        6 => Some(126),
        7 => Some(254),
        8 | _ => None //worst case for 8 bits is also out of bounds...
    }
}











pub fn static_bits_to_difference_if_allowed(bits: &[bool], max_allowed_difference: u8) -> Option<u8> {
    let encoded_difference = static_bits_to_difference(bits)?;
    if encoded_difference <= max_allowed_difference {
        Some(encoded_difference)
    } else {
        None
    }
}
pub fn static_bits_to_difference(bits: &[bool]) -> Option<u8> {
    let mut acc: u8 = 0;
    for i in 0..bits.len() {
        let single_bit_encoded: u8 = match bits[i] {
            false/*0*/ => 1,
            true /*1*/ => 2
        };

        // acc += single_bit_encoded * 2_u8.pow(i as u32); //WOULD BE THE SAME AS BELOW, BUT WITHOUT OVERFLOW HANDLING
        if let Some(influence_multiplier) = 2_u8.checked_pow(i as u32) {
            if let Some(add_summand) = single_bit_encoded.checked_mul(influence_multiplier) {
                if let Some(new_acc) = acc.checked_add(add_summand) {
                    acc = new_acc;
                    continue;
                }
            }
        }
        return None; //only arrives here if any of the previous overflow checks FAIL
    }
    return Some(acc);
}

/// Note: output bits must have length of get_num_bits_encodable(difference) , otherwise the code will raise a panic
pub fn static_difference_to_bits(mut difference: u8, output_bits: &mut [bool]) {
    for i in 0..output_bits.len() {
        if is_odd(difference) {
            output_bits[i] = false;
            difference = difference / 2;
        } else {
            output_bits[i] = true;
            difference = difference / 2 - 1;
        }
    }
}











pub fn dynamic_bits_to_difference_if_allowed(index: usize, bits: &[bool], max_allowed_difference: u8) -> Option<u8> {
    let encoded_difference = dynamic_bits_to_difference(index, bits)?;
    if encoded_difference <= max_allowed_difference {
        Some(encoded_difference)
    } else {
        None
    }
}
pub fn dynamic_bits_to_difference(index: usize, bits: &[bool]) -> Option<u8> {
    let mut rng: rand::rngs::StdRng = rand::SeedableRng::seed_from_u64(index as u64);

    let mut acc: u8 = 0;
    for i in 0..bits.len() {
        let rand = rng.gen_bool(0.5);
        let single_bit_encoded:u8 = if bits[i] ^ rand { 2 } else {1};

        // acc += single_bit_encoded * 2_u8.pow(i as u32); //WOULD BE THE SAME AS BELOW, BUT WITHOUT OVERFLOW HANDLING
        if let Some(influence_multiplier) = 2_u8.checked_pow(i as u32) {
            if let Some(add_summand) = single_bit_encoded.checked_mul(influence_multiplier) {
                if let Some(new_acc) = acc.checked_add(add_summand) {
                    acc = new_acc;
                    continue;
                }
            }
        }
        return None; //only arrives here if any of the previous overflow checks FAIL
    }
    return Some(acc);
}

/// Note: output bits must have length of get_num_bits_encodable(difference) , otherwise the code will raise a panic
pub fn dynamic_difference_to_bits(index: usize, mut difference: u8, output_bits: &mut [bool]) {
    let mut rng: rand::rngs::StdRng = rand::SeedableRng::seed_from_u64(index as u64);

    for i in 0..output_bits.len() {
        let rand = rng.gen_bool(0.5);
        let is_odd = is_odd(difference);
        output_bits[i] = !is_odd ^ rand;
        if is_odd {
            difference = difference / 2;
        } else {
            difference = difference / 2 - 1;
        }
    }
}