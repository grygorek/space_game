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

