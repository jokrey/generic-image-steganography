use crate::util::{DifCodeImage, get_length_in_bits, EncodingContainer, DifCodeError};
use crate::difference_encoder::bits_difference_converter::{dynamic_bits_to_difference, get_num_bits_decodable, dynamic_difference_to_bits, static_difference_to_bits, static_bits_to_difference, static_bits_to_difference_if_allowed};
use crate::difference_encoder::multi_bit::{decode_into_vec, encode_into_image, encode_into_vec, decode, encode, get_max_encodable_message_length_in_bits, get_min_encodable_message_length_in_bits};
use crate::difference_encoder::legacy_single_bit::{randomly_select_indices_within, encode_into_vec_at_indices, decode_into_vec_at_indices, encode_into_image_at_indices, randomly_select_indices};
use crate::difference_encoder::max_change_map_creator::{create_minimal_evenly_random_allowed_changes_map, create_minimal_evenly_random_allowed_changes_map_for, write_minimal_evenly_random_max_area_average_allowed_changes_map_for, write_minimal_evenly_random_allowed_changes_map_for};

#[test]
fn test_encode_details() {
    let message = &[0u8];
    let original = vec![0u8; 32];
    let allowed_changes_map = vec![1u8; 32];

    println!("original           : {:?}", original);
    println!("allowed_changes_map: {:?}", allowed_changes_map);

    let mut encoded = vec![123u8; 32];

    encode(message, &original, &allowed_changes_map, &mut encoded).expect("error encoding");

    println!("encoded            : {:?}", encoded);


    let allowed_changes_map = create_minimal_evenly_random_allowed_changes_map_for(message, &original, 2).expect("could not create map");

    println!("original           : {:?}", original);
    println!("allowed_changes_map: {:?}", allowed_changes_map);

    encoded = vec![123u8; 32];

    encode(message, &original, &allowed_changes_map, &mut encoded).expect("error encoding");

    println!("encoded            : {:?}", encoded);
}


#[test]
fn test_dynamic_difference_to_bits_decoder() {
    dynamic_test_difference_to_bits_decoder_for(0, &[0]);

    let mut output_bits_buffer = [false; 8];
    for index in 0..10_000 {
        dynamic_test_difference_to_bits_decoder_for_with(index, &[0], &mut output_bits_buffer);
        dynamic_test_difference_to_bits_decoder_for_with(index, &[1], &mut output_bits_buffer);
        dynamic_test_difference_to_bits_decoder_for_with(index, &[0, 0], &mut output_bits_buffer);
        dynamic_test_difference_to_bits_decoder_for_with(index, &[1, 0], &mut output_bits_buffer);
        dynamic_test_difference_to_bits_decoder_for_with(index, &[0, 1], &mut output_bits_buffer);
        dynamic_test_difference_to_bits_decoder_for_with(index, &[1, 1], &mut output_bits_buffer);

        dynamic_test_difference_to_bits_decoder_for_with(index, &[0, 0, 0], &mut output_bits_buffer);
        dynamic_test_difference_to_bits_decoder_for_with(index, &[1, 0, 0], &mut output_bits_buffer);
        dynamic_test_difference_to_bits_decoder_for_with(index, &[0, 1, 0], &mut output_bits_buffer);
        dynamic_test_difference_to_bits_decoder_for_with(index, &[0, 1, 1], &mut output_bits_buffer);
        dynamic_test_difference_to_bits_decoder_for_with(index, &[1, 1, 1], &mut output_bits_buffer);

        dynamic_test_difference_to_bits_decoder_for_with(index, &[0, 0, 0, 0], &mut output_bits_buffer);
        dynamic_test_difference_to_bits_decoder_for_with(index, &[1, 0, 0, 0], &mut output_bits_buffer);
        dynamic_test_difference_to_bits_decoder_for_with(index, &[0, 1, 0, 0], &mut output_bits_buffer);

        dynamic_test_difference_to_bits_decoder_for_with(index, &[0, 0, 0, 0, 0], &mut output_bits_buffer);
        dynamic_test_difference_to_bits_decoder_for_with(index, &[1, 1, 1, 0, 1, 0], &mut output_bits_buffer);
        dynamic_test_difference_to_bits_decoder_for_with(index, &[1, 1, 0, 1, 1, 1], &mut output_bits_buffer);
        dynamic_test_difference_to_bits_decoder_for_with(index, &[1, 1, 1, 1, 1, 1], &mut output_bits_buffer);
        dynamic_test_difference_to_bits_decoder_for_with(index, &[0, 0, 0, 0, 0, 0, 0], &mut output_bits_buffer);
        dynamic_test_difference_to_bits_decoder_for_with(index, &[0, 0, 0, 1, 0, 0, 0], &mut output_bits_buffer);
        dynamic_test_difference_to_bits_decoder_for_with(index, &[0, 0, 1, 1, 1, 0, 0], &mut output_bits_buffer);
        dynamic_test_difference_to_bits_decoder_for_with(index, &[1, 0, 0, 1, 1, 1, 0], &mut output_bits_buffer);
        dynamic_test_difference_to_bits_decoder_for_with(index, &[1, 1, 1, 1, 1, 1, 1], &mut output_bits_buffer);
    }
}
fn dynamic_test_difference_to_bits_decoder_for(index: usize, bits: &[u8]) {
    let mut output_bits_buffer = [false; 8];
    dynamic_test_difference_to_bits_decoder_for_with(index, bits, &mut output_bits_buffer);
}
fn dynamic_test_difference_to_bits_decoder_for_with(index: usize, bits: &[u8], output_bits_buffer: &mut [bool]) {
    let encoded_difference = dynamic_bits_to_difference(index, &from_bit_vec(bits)).unwrap();
    dynamic_difference_to_bits(index, encoded_difference, &mut output_bits_buffer[0..get_num_bits_decodable(encoded_difference) as usize]);
    assert_eq!(bits, &to_bit_vec(&output_bits_buffer[0..get_num_bits_decodable(encoded_difference) as usize])[..]);
}

#[test]
fn generic_test() {
    let message = &[120u8];
    let original = vec![0u8, 0u8];
    let allowed_changes = vec![255u8, 255u8];
    let mut encoded = vec![0u8, 0u8];
    encode(message, &original, &allowed_changes, &mut encoded).unwrap();
    println!("message: {:?}", message);
    println!("original: {:?}", original);
    println!("allowed_changes: {:?}", allowed_changes);
    println!("encoded: {:?}", encoded);

    let mut decoded_message = vec![];
    decode(&original, &encoded, &mut decoded_message).unwrap();
    println!("decoded_message: {:?}", decoded_message);

    assert_eq!(&message[..], &decoded_message[..]);
    for index in 20..30 {
        print_for(index, &[0]);
        print_for(index, &[1]);
        print_for(index, &[0, 0]);
        print_for(index, &[0, 1]);
        print_for(index, &[1, 0]);
        print_for(index, &[1, 1]);
        print_for(index, &[0, 0, 0, 0]);
        print_for(index, &[1, 0, 1, 0]);
        print_for(index, &[1, 0, 0, 1, 1, 1, 0]);
        println!();
        println!();
        println!();
        println!();
    }
}

fn print_for(index: usize, bits: &[u8]) {
    let encoded = dynamic_bits_to_difference(index,&from_bit_vec(bits));
    println!("-- bits: {:?}, index: {}, result: {:?}", bits, index, encoded);
    if let Some(encoded_difference) = encoded {
        let mut output_bits = vec![false; get_num_bits_decodable(encoded_difference) as usize];
        dynamic_difference_to_bits(index, encoded_difference, &mut output_bits);
        println!("---- de: {:?}", &to_bit_vec(&output_bits));
    }
    println!();
}

#[test]
fn test_static_difference_to_bits_decoder() {
    static_test_difference_to_bits_decoder_for(vec![0]);
    static_test_difference_to_bits_decoder_for(vec![1]);
    static_test_difference_to_bits_decoder_for(vec![0, 0]);
    static_test_difference_to_bits_decoder_for(vec![1, 0]);
    static_test_difference_to_bits_decoder_for(vec![0, 1]);
    static_test_difference_to_bits_decoder_for(vec![1, 1]);

    static_test_difference_to_bits_decoder_for(vec![0, 0, 0]);
    static_test_difference_to_bits_decoder_for(vec![1, 0, 0]);
    static_test_difference_to_bits_decoder_for(vec![0, 1, 0]);
    static_test_difference_to_bits_decoder_for(vec![1, 1, 0]);
    static_test_difference_to_bits_decoder_for(vec![0, 0, 1]);
    static_test_difference_to_bits_decoder_for(vec![1, 0, 1]);
    static_test_difference_to_bits_decoder_for(vec![0, 1, 1]);
    static_test_difference_to_bits_decoder_for(vec![1, 1, 1]);

    static_test_difference_to_bits_decoder_for(vec![0, 0, 0, 0]);
    static_test_difference_to_bits_decoder_for(vec![1, 0, 0, 0]);
    static_test_difference_to_bits_decoder_for(vec![0, 1, 0, 0]);
    static_test_difference_to_bits_decoder_for(vec![1, 1, 0, 0]);
    static_test_difference_to_bits_decoder_for(vec![0, 0, 1, 0]);
    static_test_difference_to_bits_decoder_for(vec![1, 0, 1, 0]);
    static_test_difference_to_bits_decoder_for(vec![0, 1, 1, 0]);
    static_test_difference_to_bits_decoder_for(vec![1, 1, 1, 0]);
    static_test_difference_to_bits_decoder_for(vec![0, 0, 0, 1]);
    static_test_difference_to_bits_decoder_for(vec![1, 0, 0, 1]);
    static_test_difference_to_bits_decoder_for(vec![0, 1, 0, 1]);
    static_test_difference_to_bits_decoder_for(vec![1, 1, 0, 1]);
    static_test_difference_to_bits_decoder_for(vec![0, 0, 1, 1]);
    static_test_difference_to_bits_decoder_for(vec![1, 0, 1, 1]);
    static_test_difference_to_bits_decoder_for(vec![0, 1, 1, 1]);
    static_test_difference_to_bits_decoder_for(vec![1, 1, 1, 1]);

    static_test_difference_to_bits_decoder_for(vec![0, 0, 0, 0, 0]);
    static_test_difference_to_bits_decoder_for(vec![1, 1, 1, 1, 1, 1]);
    static_test_difference_to_bits_decoder_for(vec![1, 1, 1, 1, 1, 1]);
    static_test_difference_to_bits_decoder_for(vec![1, 1, 1, 1, 1, 1]);
    static_test_difference_to_bits_decoder_for(vec![0, 0, 0, 0, 0, 0, 0]);
    static_test_difference_to_bits_decoder_for(vec![0, 0, 0, 1, 0, 0, 0]);
    static_test_difference_to_bits_decoder_for(vec![0, 0, 1, 1, 1, 0, 0]);
    static_test_difference_to_bits_decoder_for(vec![1, 0, 0, 1, 1, 1, 0]);
    static_test_difference_to_bits_decoder_for(vec![0, 0, 0, 0, 0, 0, 0, 0]);
}
fn static_test_difference_to_bits_decoder_for(bits: Vec<u8>) {
    let encoded_difference = static_bits_to_difference(&from_bit_vec(&bits)).unwrap();
    let mut output_bits = vec![false; get_num_bits_decodable(encoded_difference) as usize];
    static_difference_to_bits(encoded_difference, &mut output_bits);
    assert_eq!(bits, to_bit_vec(&output_bits));
}

#[test]
fn static_test_bit_to_difference_encoder() {
    assert_eq!(None, static_bits_to_difference_if_allowed(&from_bit_vec(&[0]), 0));
    assert_eq!(Some(1), static_bits_to_difference(&from_bit_vec(&[0])));
    assert_eq!(Some(2), static_bits_to_difference(&from_bit_vec(&[1])));

    assert_eq!(Some(3), static_bits_to_difference(&from_bit_vec(&[0,0])));
    assert_eq!(Some(4), static_bits_to_difference(&from_bit_vec(&[1,0])));
    assert_eq!(Some(5), static_bits_to_difference(&from_bit_vec(&[0,1])));
    assert_eq!(Some(6), static_bits_to_difference(&from_bit_vec(&[1,1])));

    assert_eq!(Some(7), static_bits_to_difference(&from_bit_vec(&[0,0,0])));
    assert_eq!(Some(8), static_bits_to_difference(&from_bit_vec(&[1,0,0])));
    assert_eq!(Some(9), static_bits_to_difference(&from_bit_vec(&[0,1,0])));
    assert_eq!(Some(10), static_bits_to_difference(&from_bit_vec(&[1,1,0])));
    assert_eq!(Some(11), static_bits_to_difference(&from_bit_vec(&[0,0,1])));
    assert_eq!(Some(12), static_bits_to_difference(&from_bit_vec(&[1,0,1])));
    assert_eq!(Some(13), static_bits_to_difference(&from_bit_vec(&[0,1,1])));
    assert_eq!(Some(14), static_bits_to_difference(&from_bit_vec(&[1,1,1])));

    assert_eq!(Some(15), static_bits_to_difference(&from_bit_vec(&[0,0,0,0])));
    assert_eq!(Some(16), static_bits_to_difference(&from_bit_vec(&[1,0,0,0])));
    assert_eq!(Some(17), static_bits_to_difference(&from_bit_vec(&[0,1,0,0])));
    assert_eq!(Some(18), static_bits_to_difference(&from_bit_vec(&[1,1,0,0])));
    assert_eq!(Some(19), static_bits_to_difference(&from_bit_vec(&[0,0,1,0])));
    assert_eq!(Some(20), static_bits_to_difference(&from_bit_vec(&[1,0,1,0])));
    assert_eq!(Some(21), static_bits_to_difference(&from_bit_vec(&[0,1,1,0])));
    assert_eq!(Some(22), static_bits_to_difference(&from_bit_vec(&[1,1,1,0])));
    assert_eq!(Some(23), static_bits_to_difference(&from_bit_vec(&[0,0,0,1])));
    assert_eq!(Some(24), static_bits_to_difference(&from_bit_vec(&[1,0,0,1])));
    assert_eq!(Some(25), static_bits_to_difference(&from_bit_vec(&[0,1,0,1])));
    assert_eq!(Some(26), static_bits_to_difference(&from_bit_vec(&[1,1,0,1])));
    assert_eq!(Some(27), static_bits_to_difference(&from_bit_vec(&[0,0,1,1])));
    assert_eq!(Some(28), static_bits_to_difference(&from_bit_vec(&[1,0,1,1])));
    assert_eq!(Some(29), static_bits_to_difference(&from_bit_vec(&[0,1,1,1])));
    assert_eq!(Some(30), static_bits_to_difference(&from_bit_vec(&[1,1,1,1])));

    assert_eq!(Some(31), static_bits_to_difference(&from_bit_vec(&[0,0,0,0,0])));
    assert_eq!(Some(32), static_bits_to_difference(&from_bit_vec(&[1,0,0,0,0])));
    assert_eq!(Some(33), static_bits_to_difference(&from_bit_vec(&[0,1,0,0,0])));
    assert_eq!(Some(34), static_bits_to_difference(&from_bit_vec(&[1,1,0,0,0])));
    assert_eq!(Some(35), static_bits_to_difference(&from_bit_vec(&[0,0,1,0,0])));
    assert_eq!(Some(36), static_bits_to_difference(&from_bit_vec(&[1,0,1,0,0])));
    assert_eq!(Some(37), static_bits_to_difference(&from_bit_vec(&[0,1,1,0,0])));
    assert_eq!(Some(38), static_bits_to_difference(&from_bit_vec(&[1,1,1,0,0])));
    assert_eq!(Some(39), static_bits_to_difference(&from_bit_vec(&[0,0,0,1,0])));
    assert_eq!(Some(40), static_bits_to_difference(&from_bit_vec(&[1,0,0,1,0])));
    assert_eq!(Some(41), static_bits_to_difference(&from_bit_vec(&[0,1,0,1,0])));
    assert_eq!(Some(42), static_bits_to_difference(&from_bit_vec(&[1,1,0,1,0])));
    assert_eq!(Some(43), static_bits_to_difference(&from_bit_vec(&[0,0,1,1,0])));
    assert_eq!(Some(44), static_bits_to_difference(&from_bit_vec(&[1,0,1,1,0])));
    assert_eq!(Some(45), static_bits_to_difference(&from_bit_vec(&[0,1,1,1,0])));
    assert_eq!(Some(46), static_bits_to_difference(&from_bit_vec(&[1,1,1,1,0])));
    assert_eq!(Some(47), static_bits_to_difference(&from_bit_vec(&[0,0,0,0,1])));
    assert_eq!(Some(48), static_bits_to_difference(&from_bit_vec(&[1,0,0,0,1])));
    assert_eq!(Some(49), static_bits_to_difference(&from_bit_vec(&[0,1,0,0,1])));
    assert_eq!(Some(50), static_bits_to_difference(&from_bit_vec(&[1,1,0,0,1])));
    assert_eq!(Some(51), static_bits_to_difference(&from_bit_vec(&[0,0,1,0,1])));
    assert_eq!(Some(52), static_bits_to_difference(&from_bit_vec(&[1,0,1,0,1])));
    assert_eq!(Some(53), static_bits_to_difference(&from_bit_vec(&[0,1,1,0,1])));
    assert_eq!(Some(54), static_bits_to_difference(&from_bit_vec(&[1,1,1,0,1])));
    assert_eq!(Some(55), static_bits_to_difference(&from_bit_vec(&[0,0,0,1,1])));
    assert_eq!(Some(56), static_bits_to_difference(&from_bit_vec(&[1,0,0,1,1])));
    assert_eq!(Some(57), static_bits_to_difference(&from_bit_vec(&[0,1,0,1,1])));
    assert_eq!(Some(58), static_bits_to_difference(&from_bit_vec(&[1,1,0,1,1])));
    assert_eq!(Some(59), static_bits_to_difference(&from_bit_vec(&[0,0,1,1,1])));
    assert_eq!(Some(60), static_bits_to_difference(&from_bit_vec(&[1,0,1,1,1])));
    assert_eq!(Some(61), static_bits_to_difference(&from_bit_vec(&[0,1,1,1,1])));
    assert_eq!(Some(62), static_bits_to_difference(&from_bit_vec(&[1,1,1,1,1])));

    println!("{:?}", static_bits_to_difference(&from_bit_vec(&[1,1,1,1,1,1])));
    println!("{:?}", static_bits_to_difference(&from_bit_vec(&[1,1,1,1,1,1,1])));
    println!("{:?}", static_bits_to_difference(&from_bit_vec(&[0,0,0,0,0,0,0,0])));
    println!("{:?}", static_bits_to_difference(&from_bit_vec(&[0,0,0,0,0,0,0,1])));
    println!("{:?}", static_bits_to_difference(&from_bit_vec(&[1,0,0,0,0,0,0,0])));
    println!("{:?}", static_bits_to_difference(&from_bit_vec(&[1,1,1,1,1,1,1,1])));
}

pub fn from_bit_vec(bits: &[u8]) -> Vec<bool> {
    let mut bool_bits = Vec::with_capacity(bits.len());
    for bit in bits {
        bool_bits.push(if *bit == 0 {false} else {true});
    }
    bool_bits
}
pub fn to_bit_vec(bits: &[bool]) -> Vec<u8> {
    let mut bool_bits = Vec::with_capacity(bits.len());
    for bit in bits {
        bool_bits.push(if *bit {1} else {0});
    }
    bool_bits
}



#[test]
fn test_bytes() {
    let message_bytes = vec![1u8, 123u8, 98u8];
    let original: Vec<u8> = (0..500).map(|x| x as u8).collect();
    println!("message        : {:?}", message_bytes);
    println!("original       : {:?}", original);

    let mut allowed_changes: Vec<u8> = create_minimal_evenly_random_allowed_changes_map(&message_bytes, &original).expect("could not create map");
    println!("allowed_changes: {:?}", allowed_changes);

    let encoded = encode_into_vec(&message_bytes, &original, &mut allowed_changes).unwrap();
    println!("encoded        : {:?}", encoded);

    let decoded = decode_into_vec(&original, &encoded).unwrap();
    println!("decoded        : {:?}", decoded);

    assert_eq!(message_bytes, decoded);
}

#[test]
fn test_multi_bit_encoding() {
    let message_bytes: Vec<u8> = (44..51).map(|x| x as u8).collect();
    let original: Vec<u8> = (100..110).map(|x| x as u8).collect();
    println!("message        : {:?}", message_bytes);
    println!("original       : {:?}", original);

    let mut allowed_changes: Vec<u8> = create_minimal_evenly_random_allowed_changes_map(&message_bytes, &original).expect("could not create map");
    println!("allowed_changes: {:?}", allowed_changes);

    let encoded= encode_into_vec(&message_bytes, &original, &mut allowed_changes).unwrap();
    println!("encoded        : {:?}", encoded);

    let decoded = decode_into_vec(&original, &encoded).unwrap();
    println!("decoded        : {:?}", decoded);

    assert_eq!(message_bytes, decoded);
}


#[test]
fn test_image_without_save() {
    let message_bytes: Vec<u8> = (0..64).map(|_| { rand::random::<u8>() }).collect();
    let original_image_path = "test/RealisticTestImage.jpg";

    println!("message bytes: {:?}", message_bytes);
    println!("original image path: {}", original_image_path);

    let original_image = DifCodeImage::open(original_image_path).unwrap();

    let allowed_changes: Vec<u8> = create_minimal_evenly_random_allowed_changes_map(&message_bytes, &original_image).expect("could not create map");
    println!("allowed_changes len: {:?}", allowed_changes.len());

    let encoded_image = encode_into_image(&message_bytes, &original_image, &allowed_changes).expect("encoding failed");

    let decoded_message = decode_into_vec(&original_image, &encoded_image).unwrap();

    assert_eq!(message_bytes, decoded_message);
}


#[test]
fn test_image_with_save() {
    let message_bytes: Vec<u8> = (0..64).map(|_| { rand::random::<u8>() }).collect();
    let original_image_path = "test/RealisticTestImage.jpg";
    let encoded_image_path = "test/RealisticTestImage.png";

    println!("message bytes: {:?}", message_bytes);
    println!("original image path: {}", original_image_path);

    let original_image = DifCodeImage::open(original_image_path).unwrap();

    let encoded_image = encode_into_image(&message_bytes, &original_image,
                                          &create_minimal_evenly_random_allowed_changes_map(&message_bytes, &original_image).expect("failed to select indices")
    ).expect("encoding/saving failed");
    encoded_image.save(encoded_image_path).expect("saving image failed");

    let decoded_message_from_ram_data = decode_into_vec(&original_image, &encoded_image).unwrap();
    assert_eq!(message_bytes, decoded_message_from_ram_data);

    //"proof" of commutativity
    let decoded_message_from_ram_data_commutativity = decode_into_vec(&encoded_image, &original_image).unwrap();
    assert_eq!(message_bytes, decoded_message_from_ram_data_commutativity);


    //time ----


    let original_image_reloaded = DifCodeImage::open(original_image_path).unwrap();
    assert_eq!(original_image, original_image_reloaded);

    let encoded_image_reloaded = DifCodeImage::open(encoded_image_path).unwrap();
    assert_eq!(encoded_image, encoded_image_reloaded);

    let decoded_message_from_disk = decode_into_vec(&original_image_reloaded, &encoded_image_reloaded).unwrap();
    assert_eq!(message_bytes, decoded_message_from_disk);
}

#[test] #[ignore]
fn test_image_multi_bit_re_max_diff_with_save() {
    let original_image_path = "test/RealisticTestImage.jpg";
    let encoded_image_path = "test/RealisticTestImageMultiBit.png";

    println!("original image path: {}", original_image_path);

    let original_image = DifCodeImage::open(original_image_path).unwrap();

    let message_bytes: Vec<u8> = (0..(original_image.len() as f64*0.7) as usize).map(|_| { rand::random::<u8>() }).collect(); //~2 bit per rgb value part
    println!("message length: {:?}", message_bytes.len());

    let encoded_image = encode_into_image(&message_bytes, &original_image,
                                          &create_minimal_evenly_random_allowed_changes_map(&message_bytes, &original_image).expect("failed to select indices")
    ).expect("encoding/saving failed");
    encoded_image.save(encoded_image_path).expect("saving image failed");

    let decoded_message_from_ram_data = decode_into_vec(&original_image, &encoded_image).unwrap();
    assert_eq!(message_bytes, decoded_message_from_ram_data);

    //time ----

    let original_image_reloaded = DifCodeImage::open(original_image_path).unwrap();
    assert_eq!(original_image, original_image_reloaded);

    let encoded_image_reloaded = DifCodeImage::open(encoded_image_path).unwrap();
    assert_eq!(encoded_image, encoded_image_reloaded);

    let decoded_message_from_disk = decode_into_vec(&original_image_reloaded, &encoded_image_reloaded).unwrap();
    assert_eq!(message_bytes, decoded_message_from_disk);
}

#[test] #[ignore]
fn test_image_multi_bit_max_av_area_diff_with_save_MAX_ENCODE() {
    let original_image_path = "test/RealisticTestImage.jpg";
    let max_area_av_encoded_image_path = "test/RealisticTestImageMultiBit_MaxAreaAvDiff_MAXMAX.png";
    let re_encoded_image_path_of_same_message_length = "test/RealisticTestImageMultiBit_MaxAreaAvDiff_MAXMAX_RE_ENCODED_COMPARISION.png";

    println!("original image path: {}", original_image_path);

    let original_image = DifCodeImage::open(original_image_path).unwrap();

    let mut message_bytes: Vec<u8> = (0..original_image.len()).map(|_| { rand::random::<u8>() }).collect(); //~2 bit per rgb value part
    println!("message length: {:?}", message_bytes.len());

    let mut allowed_changes_map = vec![0u8; original_image.len()];
    let selection_result = write_minimal_evenly_random_max_area_average_allowed_changes_map_for(&message_bytes, &original_image, &mut allowed_changes_map);
    println!("selection_result: {:?}", selection_result);
    let possible_size = match selection_result {
        Ok(_) => message_bytes.len(),
        Err(dif_error) => match dif_error {
            DifCodeError::InternalCapacityReached(num_successful) => num_successful,
            _ => 0
        }
    };
    println!("possible_size: {:?}", possible_size);
    message_bytes.truncate(possible_size/8);
    println!("message_bytes len: {:?}", message_bytes.len());

    let encoding_result = encode_into_image(&message_bytes, &original_image, &allowed_changes_map);
    println!("encoding_result: {:?}", encoding_result);
    let encoded_image = match encoding_result {
        Ok(image) => image,
        Err(dif_error) => match dif_error {
            DifCodeError::InternalCapacityReached(num_successful) => {
                message_bytes.truncate(num_successful/8);
                println!("message_bytes len: {:?}", message_bytes.len());
                println!("num_successful: {:?}", num_successful);
                encode_into_image(&message_bytes, &original_image, &allowed_changes_map).expect("encoding failed")
            },
            _ => panic!("unexpected error")
        }
    };
    encoded_image.save(max_area_av_encoded_image_path).expect("saving image failed");

    let decoded_message_from_ram_data = decode_into_vec(&original_image, &encoded_image).unwrap();
    assert_eq!(message_bytes, decoded_message_from_ram_data);

    //time ----

    let original_image_reloaded = DifCodeImage::open(original_image_path).unwrap();
    assert_eq!(original_image, original_image_reloaded);

    let encoded_image_reloaded = DifCodeImage::open(max_area_av_encoded_image_path).unwrap();
    assert_eq!(encoded_image, encoded_image_reloaded);

    let decoded_message_from_disk = decode_into_vec(&original_image_reloaded, &encoded_image_reloaded).unwrap();
    assert_eq!(message_bytes, decoded_message_from_disk);



    //CREATE AND VALIDATE COMPARISON IMAGE

    let image_encoded_using_re = encode_into_image(&message_bytes, &original_image, &create_minimal_evenly_random_allowed_changes_map(&message_bytes, &original_image).expect("failed to select re")).expect("could not encode using evenly random");
    image_encoded_using_re.save(re_encoded_image_path_of_same_message_length).expect("could not save comparison image");

    let decoded_message_from_re_encoded = decode_into_vec(&original_image, &image_encoded_using_re).unwrap();
    assert_eq!(message_bytes, decoded_message_from_re_encoded);
}

#[test]// #[ignore]
fn test_image_multi_bit_re_diff_with_save_MAX_ENCODE() {
    let original_image_path = "test/RealisticTestImage.jpg";
    let encoded_image_path = "test/RealisticTestImageMultiBit_reDiff_MAXMAX.png";

    println!("original image path: {}", original_image_path);

    let original_image = DifCodeImage::open(original_image_path).unwrap();

    let mut message_bytes: Vec<u8> = (0..original_image.len()).map(|_| { rand::random::<u8>() }).collect(); //~2 bit per rgb value part
    println!("message length: {:?}", message_bytes.len());

    let mut allowed_changes_map = vec![0u8; original_image.len()];
    let selection_result = write_minimal_evenly_random_allowed_changes_map_for(&message_bytes, &original_image, &mut allowed_changes_map);
    println!("selection_result: {:?}", selection_result);
    let possible_size_in_bits = match selection_result {
        Ok(_) => message_bytes.len(),
        Err(dif_error) => match dif_error {
            DifCodeError::InternalCapacityReached(num_bits_successful) => num_bits_successful,
            _ => 0
        }
    };
    println!("possible_size: {:?}", possible_size_in_bits);
    message_bytes.truncate(possible_size_in_bits / 8);

    let encoding_result = encode_into_image(&message_bytes, &original_image, &allowed_changes_map);
    println!("encoding_result: {:?}", encoding_result);
    let encoded_image = match encoding_result {
        Ok(image) => image,
        Err(dif_error) => match dif_error {
            DifCodeError::InternalCapacityReached(num_bits_successful) => {
                message_bytes.truncate(num_bits_successful / 8);
                encode_into_image(&message_bytes, &original_image, &allowed_changes_map).expect("encoding/saving failed")
            }
            _ => panic!("unexpected error")
        }
    };
    encoded_image.save(encoded_image_path).expect("saving image failed");

    let decoded_message_from_ram_data = decode_into_vec(&original_image, &encoded_image).unwrap();
    assert_eq!(message_bytes, decoded_message_from_ram_data);

    //time ----

    let original_image_reloaded = DifCodeImage::open(original_image_path).unwrap();
    assert_eq!(original_image, original_image_reloaded);

    let encoded_image_reloaded = DifCodeImage::open(encoded_image_path).unwrap();
    assert_eq!(encoded_image, encoded_image_reloaded);

    let decoded_message_from_disk = decode_into_vec(&original_image_reloaded, &encoded_image_reloaded).unwrap();
    assert_eq!(message_bytes, decoded_message_from_disk);
}

// #[test]
// fn test_image_multi_bit_with_display() {
//     let original_image_path = "test/RealisticTestImage.jpg";
//
//     println!("original image path: {}", original_image_path);
//
//     let original_image = DifCodeImage::open(original_image_path).unwrap();
//
//     let message_bytes: Vec<u8> = (0..original_image.len()/2).map(|_| { rand::random::<u8>() }).collect(); //~2 bit per rgb value part
//     println!("message bytes: {:?}", message_bytes);
//
//     let encoded_image = encode_into_image(&message_bytes, &original_image,
//                                           &create_minimal_evenly_random_allowed_changes_map_for(&message_bytes, &original_image)
//     ).expect("encoding/saving failed");
//
//     let decoded_message_from_ram_data = decode_into_vec(&original_image, &encoded_image).unwrap();
//     assert_eq!(message_bytes, decoded_message_from_ram_data);
//
//
//     let pool = ThreadPool::new(2);
//     pool.execute(move || display_image("Original", &original_image.raw()));
//     pool.execute(move || display_image("Encoded", &encoded_image.raw()));
//     pool.join();
// }





#[test]
fn test_bytes_single_bit() {
    let message_bytes = vec![0u8, 1u8, 2u8];
    let original: Vec<u8> = (0..1024).map(|x| x as u8).collect();
    println!("message: {:?}", message_bytes);
    println!("original len: {:?}", original.len());

    let mut selected_indices = randomly_select_indices_within(&message_bytes, &original);
    println!("selected_indices: {:?}", selected_indices);

    let encoded= encode_into_vec_at_indices(&message_bytes, &original, &mut selected_indices).unwrap();
    println!("encoded len: {:?}", encoded.len());

    let decoded = decode_into_vec_at_indices(&original, &encoded).unwrap();
    println!("decoded len: {:?}", decoded.len());

    assert_eq!(message_bytes, decoded);
}


#[test]
fn test_image_without_save_single_bit() {
    let message_bytes: Vec<u8> = (0..64).map(|_| { rand::random::<u8>() }).collect();
    let original_image_path = "test/RealisticTestImage.jpg";

    println!("message bytes: {:?}", message_bytes);
    println!("original image path: {}", original_image_path);

    let original_image = DifCodeImage::open(original_image_path).unwrap();

    let encoded_image = encode_into_image_at_indices(&message_bytes, &original_image,
                                                     &randomly_select_indices(get_length_in_bits(&message_bytes), original_image.len())
    ).expect("encoding failed");

    let decoded_message = decode_into_vec_at_indices(&original_image, &encoded_image).unwrap();

    assert_eq!(message_bytes, decoded_message);
}


#[test]
fn test_image_with_save_single_bit() {
    let message_bytes: Vec<u8> = (0..64).map(|_| { rand::random::<u8>() }).collect();
    let original_image_path = "test/RealisticTestImage.jpg";
    let encoded_image_path = "test/RealisticTestImageSingleBit.png";

    println!("message bytes: {:?}", message_bytes);
    println!("original image path: {}", original_image_path);

    let original_image = DifCodeImage::open(original_image_path).unwrap();

    let encoded_image = encode_into_image_at_indices(&message_bytes, &original_image,
                                                     &randomly_select_indices(get_length_in_bits(&message_bytes), original_image.len())
    ).expect("encoding/saving failed");
    encoded_image.save(encoded_image_path).expect("saving image failed");

    let decoded_message_from_ram_data = decode_into_vec_at_indices(&original_image, &encoded_image).unwrap();
    assert_eq!(message_bytes, decoded_message_from_ram_data);

    //"proof" of commutativity
    let decoded_message_from_ram_data_commutativity = decode_into_vec_at_indices(&encoded_image, &original_image).unwrap();
    assert_eq!(message_bytes, decoded_message_from_ram_data_commutativity);


    //time ----


    let original_image_reloaded = DifCodeImage::open(original_image_path).unwrap();
    assert_eq!(original_image, original_image_reloaded);

    let encoded_image_reloaded = DifCodeImage::open(encoded_image_path).unwrap();
    assert_eq!(encoded_image, encoded_image_reloaded);

    let decoded_message_from_disk = decode_into_vec_at_indices(&original_image_reloaded, &encoded_image_reloaded).unwrap();
    assert_eq!(message_bytes, decoded_message_from_disk);
}


#[test]
fn test_max_encodable_message_length() {
    let message_bytes: Vec<u8> = (0..1).map(|_| { rand::random::<u8>() }).collect();
    let original: Vec<u8> = (0..8).map(|x| x as u8).collect();

    let allowed_changes_map = create_minimal_evenly_random_allowed_changes_map_for(&message_bytes, &original, 2).expect("failed to select indices");
    let min_encodable = get_min_encodable_message_length_in_bits(&original, &allowed_changes_map);
    let max_encodable = get_max_encodable_message_length_in_bits(&original, &allowed_changes_map);

    println!("message_bytes: {:?}", message_bytes);
    println!("original           : {:?}", original);
    println!("allowed_changes_map: {:?}", allowed_changes_map);
    println!("min_encodable: {}", min_encodable);
    println!("max_encodable: {}", max_encodable);
    assert_eq!(min_encodable, max_encodable);
    assert_eq!(8, max_encodable);


    let message_bytes: Vec<u8> = (0..1).map(|_| { rand::random::<u8>() }).collect();
    let original: Vec<u8> = (0..4).map(|x| x as u8).collect();
    println!("message_bytes: {:?}", message_bytes);
    println!("original           : {:?}", original);

    let allowed_changes_map: Vec<u8> = (5..9).map(|x| x as u8).collect(); //distance of 5 each index
    let min_encodable = get_min_encodable_message_length_in_bits(&original, &allowed_changes_map);
    let max_encodable = get_max_encodable_message_length_in_bits(&original, &allowed_changes_map);

    println!("allowed_changes_map: {:?}", allowed_changes_map);
    println!("min_encodable: {}", min_encodable);
    println!("max_encodable: {}", max_encodable);
    assert_ne!(min_encodable, max_encodable);
    assert_eq!(4, min_encodable);
    assert_eq!(8, max_encodable);

    //NOT NECESSARILY POSSIBLE::
    // let mut encoded: Vec<u8> = vec![0u8;original.len()];
    // encode(&message_bytes, &original, &allowed_changes_map, &mut encoded).expect("encoding failed");
    //
    // println!("encoded            : {:?}", encoded);
    //
    // let mut decoded = Vec::new();
    // decode(&original, &encoded, &mut decoded).expect("decoding failed");
    //
    // println!("decoded      : {:?}", decoded);
}