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

