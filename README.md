# Generic Image Steganography

Simple Concept for steganographically encoding generic bytes into images


**Idea:**

Alice and Bob have shared images (like most people) and Alice wants to send Bob a message.
Alice selects an image (by some code), enters a message and the image into the encoder.
The encoder outputs a second image looking identical to the first one (to the human eye).

It can be sent across pretty much any digital medium without raising an eyebrow.

Bob enters the original image and the encoded image into the decoder.
The decoder outputs the message.


**User-Steps (in the provided UI):**

    - Choose between encoding/decoding
       - ENCODING:
          - Choose message (Choose between utf8/base64 -> Enter message (encoded))
          - Choose encryption (Choose between aes/(cancel, i.e. none is allowed) -> Enter password)
          - Choose original image (Choose between url/path -> Enter url/path)
          - Choose output image path (Enter path)
       - DECODING:
          - Choose original/encoded image (Choose between url/path -> Enter url/path)
          - Choose encoded/original image (Choose between url/path -> Enter url/path)
          - Choose decryption (Choose between aes/(cancel, i.e. none is allowed) -> Enter password)
          - Choose decoding (Choose between utf8/base64)


**Notes:**

	The cryptographic security will not be particularly high given some constraints.
	For example access to the shared images and the given encoded image, breaks everything.
		(In that case the images can simply be checked for equality/similarity to find the 'key')
	The security lies in the obscurity of the transfer method(yeah, yeah security by obscurity is bad - but in real life,
	        non-mass applications it provides a barely surmountable obstacle), the number of possible keys compared to the length of the message and,
	            most importantly, the impossibility to distinguish between a normal image and an image with an encoded message (unless, of course, the original image is known).
	
	The following should hold: Without the original image the message cannot be decoded (i.e. just 1 bit difference per pixel (which should not be detectable even by analyzers)).

    The practical applications of this are limited to some very specific and unlikely use cases, but it will help with further studying Rust.


**Algorithm:**

    Encode:
        The message provided in bytes is decoded into bits (big endian). 'n' shall denote the number of bits.
            The message can be encoded into bytes using any format (utf8, base64, others)
            The message can also be encrypted using any algorithm (AES, 3DES, others)
        In the given byte container each byte is assigned a maximum change by some metric
            (randomly or evenly spaced or so that the encoded data still resembles the original data)
            Note: If the container is an image the bytes are the rgb values of each pixel, addressed as if the image was a flat stored 3D matrix (width x height x 3).
                  R, G and B channel are used independently to encode data.
        For each bit (or a number of sequential bits) a change to a byte in the original container is made
            The direction of the change does not matter and is chosen based on the maximum allowed change at that byte chosen by the metric.
            Which bit string results in which magnitude of the change is deterministic, but pseudo random (seeded rng).
            (The direction is not used for encoding due to problems that would arise on dark and bright images
                (that contain many 254s, 255s, 0s and 1s),
                additionally it provides commutativity between encoded and original image in the decode step).
        Done
    Decode:
        The algorithm is provided with two byte containers, one shall be the original container used in encoding, the other shall be the result of the encoding.
            For example a simple byte array or an image.
        Changes between the two containers are detected.
        From the changes the original message is restored based on the mutating code table (implemented from a seeded rng).
        The resulting byte array is the original message in bytes
            The message can also be decrypted using any algorithm (AES, 3DES, others)
            The message can be decoded from bytes using any format (utf8, base64, others)
        Done



**Possible improvements:**

    - Select indices based on metric.
      Analyze variance of overlapping areas, weight each pixel/index based on the variance in its areas.
      Additionally weight each pixel based on its distance to previously selected pixels
           (high distance to previously selected pixels -> higher probability of selection)
    - Make the direction of the change dependent on the average rgb vector in the area.
    (- include alpha channel
        (requires much more complex metrics for selection, because alpha is typically the same for large areas and should not be changed then)