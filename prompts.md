--------------------------------
Prompt: __Update my application to open graphical window in full screen mode and full HD resulution. Make yellow pixels at each corner and draw a 20x20px red square in the centre.__

It generated some code which was failing. After feeding back errors it found a solution.

--------------------------------
Prompt: __Looking good. Can you change screen resolution to the Full HD?__

No problems here.

--------------------------------
Prompt: _Add cursors control of that square. Using my keyboard cursors I am able to move the square around the screen._

Prompt: _Refactor the code. Move the key handling to separate functions._

Prompt: _Move the square a bit faster_

--------------------------------
Prompt: Can you improve keyboard handling so when I move my square with keyboards it does not stop moving when I press another keyboard (e.g. space). Something like a multikey support.

Prompt: It is not fully working. If i hold Left and press space, the movement stops until I press space again. Seems like redraw is called only when a key event comes.

Prompt: The main function is very large and handling different operations. Refactor the code. Split on functions and structures if needed.

--------------------------------

Prompt: The black background is the interstellar space. Please generate stars in the background. The largest stars are max 5pixels in diameter and the smallest 1pixel. Some are faint bluish, other graish, other yellowish and other reddish. Max count of stars 20 but can be less.

Prompt: Make the stars animated falling slowly from the top of the screen. It will feel like my square is moving in the space.

Prompt: The stars are moving only when my square is moving. Please make the stars moving regardless of the square movement.

--------------------------------

Prompt: Remove the corner yellow pixels. Move the stars related code out of main.rs to a dedicated file.

(Running out of tokens at this stage. Starting new chat.)

Prompt: Our context: We build together a Rust game like desktop application. There are stars falling down from the top of the screen and black background. There is a red square that I can control its movement with cursors. Hitting other keys does not interrupt the movement. Do not modify any of the files yet.


Prompt: The stars generation and drawing code is in stars.rs. But I think it is duplicated in main.rs too. Can you remove that code from main.rs please.

Prompt: Remove drawing yellow pixels in the corner of the screen.

Prompt: Move update_stars function from main.rs to stars.rs

--------------------------------

Prompt: There is few build warnings. Are they fixeable?
cargo build warning: unused import: MAX_STARS --> src\main.rs:12:57 | 12 | use stars::{generate_stars, draw_star, SimpleRng, Star, MAX_STARS}; |                                                         ^^^^^^^^^ | = note: #[warn(unused_imports)] on by default
warning: unused Result that must be used --> src\main.rs:155:17 | 155 |                 self.pixels.resize_surface(self.size.width, self.size.height); |                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ | = note: this Result may be an Err variant, which should be handled = note: #[warn(unused_must_use)] on by default help: use let _ = ... to ignore the resulting value | 155 |                 let _ = self.pixels.resize_surface(self.size.width, self.size.height); |                 +++++++
warning: unused Result that must be used --> src\main.rs:161:17 | 161 |                 self.pixels.resize_surface(self.size.width, self.size.height); |                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ | = note: this Result may be an Err variant, which should be handled help: use let _ = ... to ignore the resulting value | 161 |                 let _ = self.pixels.resize_surface(self.size.width, self.size.height); |                 +++++++

--------------------------------

Prompt: I would like to replace the square with an image of a spaceship. In what format of the file would be best to include picture of that spaceship in this application?

It suggested:

Use a PNG with RGBA (8-bit per channel).

I went to Google Gemini and generated an image with this prompt:
I am writing a game where I control a spaceship pointing towards top of the screen. Can you generate PNG image of my spaceship. Image 100x100 pixels

The image was actually not 100x100 and did not have trasparent background. I went to Gimp and removed the background and resized the image. Not exaclty 100x100. Just from edge to edge of the spaceship.

Next I downloaded the generated image to png/ship.png

Prompt: There is ship.png file in the 'png' subdirectory. Add code to load that file and replace my red square with the loaded ship image.

--------------------------------

Prompt: Create a new file called drawing.rs and move 'blend_pixel', set_pixel to that new file. Also move ship sprite drawing code to a dedicated function. I think this function would be generic for different sprites. Am I right? Then move it to drawing.rs too.

Prempt: Rename 'square' to 'ship'

--------------------------------

I asked Gemini to generate a laser beam. It looked good. I removed background and resized to 5x20pix. Added as beam.png.

--------------------------------

Prompt: Improve performance. Improve ship clipping at bottom of the screen.

It was still choppy. By telling it what is going on, it finally improved to the acceptable level.

--------------------------------

Prompt: Wait. I can see there were no optimizations done in drawing.rs. Can you optimize it?

--------------------------------

Prompt: Performance is looking good now.
There is a file beam.png in png directory. Can you use it to generate my spaceship to shoot a laser beam?
I want one shot per one space press. Autofire will be a future bonus. Ignore the bonus now.

Prompt: Generate an image of an enemy ship. View from top, ship pointing 'south', transparent background.

Google Gemini. As usual, edit in Gimp, remove background, crop and resize.

Prompt: Remove mouse cursor and place my spaceship (the one I control) at 1/5th of the screen at bottom, instead of centre as is now.

--------------------------------

Prompt: Using enemy1.png file, show 3 rows, 8 ships in each row. Keep about half of the ship width distance between them.
