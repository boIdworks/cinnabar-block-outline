# Feature reference

Everything Boldy Mods does, in the order of the menu's start page. The short version is in the
[README](../README.md); the jump reset trainer has its own page, [jump-reset.md](jump-reset.md);
the settings file and permissions are in [settings.md](settings.md).

## Contents

- [Targeting](#targeting)
- [Exact shapes](#exact-shapes)
- [Aimed mob](#aimed-mob)
- [Bridging](#bridging)
- [Thickness](#thickness)
- [Fill](#fill)
- [Menu](#menu)
- [Fullbright](#fullbright)
- [Arrow counter](#arrow-counter)
- [CPS counter](#cps-counter)
- [FPS counter](#fps-counter)
- [Crosshair](#crosshair)
- [Zoom](#zoom)
- [Hides with the HUD](#hides-with-the-hud)
- [Mobs and players](#mobs-and-players)

## Targeting

The mod raycasts from your eye against the blocks' selection boxes, so slabs, stairs and other partial blocks get their real outline. Only edges of faces turned towards the camera are drawn, so you never see edges hidden behind the block.

## Exact shapes

Stairs, fences, walls and other blocks made of several boxes are outlined along their real shape (up to 4 boxes; bigger shapes get their bounding box), with no lines across flat faces; the fill and the game-highlight fix follow the shape too.

## Hotbar keys

Each hotbar slot shows its assigned keyboard key at the top centre. The labels follow Cinnabar's keyboard bindings, including modifiers; unbound slots remain blank. The layout follows the standard centred, 182-unit hotbar, so labels may sit off the slots at an unusual HUD layout. They show only in a world, and switch off with **F8**, **Vanilla HUD**, **Hotbar keys** (on by default). Off, the mod does no work for them.

## Effect timers

Under each status-effect icon the remaining time shows as `m:ss` (`inf` for an effect with no end). Cinnabar is polled five times a second, and the labels are sent only when a time actually changed. The timers sit in the same two rows as the game's icons (good effects in one, bad in the other, sixteen to a row) and assume the host lists effects in the order it draws them. Switch them off with **F8**, **Vanilla HUD**, **Effect timers** (on by default). Off, or outside a world, no host call is made.

## Vanilla HUD

**F8**, **Vanilla HUD** has the hunger row's switch, plus the two switches above. Cinnabar's mod API does not currently expose independent hiding of hearts, armor icons, or XP.

## Armor HUD

The armor display is off until you switch it on with the **Armor HUD on** switch on the start page. Once on, it always shows its four slots, even with nothing equipped. Each slot is an inset, half-transparent black square in a thin grey bevelled frame. It shows equipped items and a coloured durability bar when durability is available. Open **F8**, **Armor HUD** to open the shared HUD editor, switch its layout, hide durability percentages, or show hotbar-style slot backgrounds. These options are saved. The armor HUD can be moved and resized in the shared editor.

## Aimed mob

A mob counts as aimed exactly when Cinnabar says your crosshair would hit it (attack reach, blocks in the way), not by the mod's own guess.

## Bridging

When the look ray hits nothing and you look steeply down (Cinnabar's bridging rule: look direction Y below −0.7, support within 2 blocks under your eyes), the block you stand on is outlined if your crosshair points into one of the eight gaps around it, sides or corners.

## Thickness

Lines keep the chosen thickness within 2.5 blocks and thin out with distance beyond that, never below 1 pixel.

## Fill

An optional see-through fill in the outline colour over the block's visible faces.

## Menu

A fresh install starts with every feature switched off (outlines, counters, zoom, jump reset, crosshair, fullbright, hotbar keys, effect timers and the auto queue), so you choose what to turn on. Press **F8** in game (the menu key, changeable in the menu: click **Menu key**, then press a key; Esc cancels). Esc closes it. Rest the mouse on any control for a short explanation. Buttons are 18 units tall; every on/off setting is a pill switch (knob right and purple = on) the sliders are slim bars with a round knob, each page's settings sit in lighter inset panels under small-capital headings, the start page's buttons carry drawn icons (a block, a mob in its hitbox, an arrow, a magnifier) in the page's colour, key buttons look like keys (a button that is not set says **Click to set**), a value that is not its default says **reset** beside it (click it to reset), and the counter guide shares the menu's look; buttons that do something (Reset, Move and resize) have their caption in the middle. The bottom line of the card shows the help of whatever the pointer or the keyboard is on straight away; the floating box follows after a short rest.

- **Keyboard:** the whole menu works without the mouse. The **arrow keys** move a thin frame the way they point: **Up** / **Down** to the control above or below (staying in the same column, so Rainbow goes down to Match fill and Pulse to Reset), **Left** / **Right** to the one beside it (along the swatches, the saved colours and side-by-side buttons); past the last control they wrap round, and the first arrow press only brings the frame in. On a **slider** or a value with **-** / **+**, Left and Right change it instead (by 5, or 0.5 px for thickness; hold **Shift** for five times that), and **holding** the key repeats, so you can sweep a slider; on a switch with nothing beside it they turn it off / on. **Enter** or **Space** presses the focused control (on a slider it starts typing the number; on a value with **-** / **+** it resets the value). **Tab** and **Shift+Tab** walk every control in reading order, **Home** / **End** jump to the first and last, **Backspace** goes back a page and **Esc** closes the menu. Clicking with the mouse puts the frame away. The keys do nothing while a box is typed into or a key is awaited. The reset buttons ask **Sure? Click again** and take the question back after 4 seconds, or when you move on.
- Every mod has an on/off switch on the start page, right of its button (**block_on** and **mob_on** are saved; the others use their own settings). A mod that is off draws nothing, and its settings are kept.
- The menu opens on a start page: pick **Block outline**, **Mob hitbox**, **Arrow counter**, **CPS counter**, **FPS counter**, **Zoom**, **Lighting**, **Jump reset**, **Crosshair** or **Hive Utilities**. Each page has its own colours, opacity, thickness, fill and rainbow (pulse, and a separate fill colour, are for blocks only: mob hitboxes have no fill; the Mob hitbox page has **Line colour** and **Aimed colour** at the top instead); **< Back** (top right) goes back one step and **X** beside it closes the menu.
- **Line colour / Fill colour** show the colour tools, which stay hidden until then and replace the settings in the same narrow menu, so the middle of the screen stays clear (click **< Back**, or the same button again, to go back). They edit that colour: the Red, Green and Blue bars, the wheel (hue and saturation, keeping brightness), the hex code field (click it and type 0-9 and A-F; Enter applies, Escape cancels, invalid codes are dropped), the quick colours and the saved colours.
- Drag a bar, or click the number beside it and type an exact value (Enter or a click elsewhere applies, Escape cancels, Backspace deletes). The mod reads these keys itself, because Cinnabar's native text editor does not open for this menu in game; copy and paste are therefore not available. Whatever is under the mouse is highlighted.
- **Opacity, Thickness, Fill** set the line opacity, up-close line width and face fill strength (the fill hides while you hold your attack key to break a block, so the cracks stay visible; the mod reads your key bindings, so a remapped attack key works too). **Match fill** makes the fill follow the line colour (rainbow included); turn it off to give the fill its own colour; while it is on, the Fill colour button is hidden and **Line & fill colour** stretches across its place.
- **Rainbow** cycles the line colour; **Pulse** gently fades the outline in and out.
- **Saved colours:** **Save colour** stores the colour being edited in the next of four slots (oldest replaced); click a slot to use it.
- **Reset page** restores that page's defaults, keeping the other page, the menu key and saved colours. All three reset buttons (page, counter, zoom) turn red and ask **Sure? Click again** first; clicking anything else cancels. The menu key is set on the start page, under **Menu** (**Fullbright** and its key are under **Lighting**); **Hide game highlight** (the old *Game highlight fix*) is on the Block outline page.
- The outline stays visible while the menu is open, and every change is saved.

## Fullbright

The **Lighting** page's **Fullbright** switch (and its key) uses Cinnabar's own local fullbright, so caves and nights look like daylight on your screen only (nothing is sent to the server). Like the Simple Fullbright mod, it asks Cinnabar for fullbright every frame, so it stays on. **Fullbright key** (start page, under Lighting, not set by default) switches it on and off in game with an on-screen message: click it, then press a key (Escape cancels; Backspace or Delete unbinds it). It needs the **Fullbright** permission.

The **Hive Utilities** page (the last button on the start page) holds **Auto queue**, which is off by default and queues the next Hive game for you. While it is on, the mod reads the chat four times a second. Hive's `Finding you a game of BED-DUOS...` line, which appears whenever you queue, names the queue exactly as `/q` takes it, so the mod remembers it. It sends `/q <mode>` (for example `/q BED-DUOS`) one second after any of these:

- `» Game OVER!`.
- `All members of your party are now spectating.` (your whole party is out, so you can leave).
- You are eliminated **and not in a party**, whatever the mode (a solo player in Duos or Squads is queued as soon as they are out). Being eliminated is Hive's own `You were eliminated`; or `You were killed by ...` in a game with no respawns (Skywars and Survival Games, any size; in Bedwars the same kind of line can mean a respawn, so it is not used there); or your own life state turning off with a `FINAL KILL!` line within a second (dying with your bed still standing is not a `FINAL KILL!`, so it does not count, and no player name is needed). The mod then sends `/p list`: Hive's reply `You have to be in a party to run this command.` means you are alone, so it queues. Any other reply, or none within three seconds, is taken as a party, and it waits for the party line instead, so it never leaves teammates who are still playing.

A message on the HUD says what it did (`Auto queue: on, watching chat`, `Auto queue: eliminated, checking your party`, `Auto queue: you died` when a death was seen but was not final, `Auto queue: /q SKY`). The mode is the one Hive last named in a `Finding you a game of ...` line, and after the mod queues a game itself it stays the same, because Hive puts you straight into the next game without that line. So queue once by hand (or with `/q`) and it carries on from there; a queue you start by hand later names its own mode. With no mode known it queues nothing (Hive has no `/rq`) and says so on the HUD. After a game you reach some other way (the hub menus), queue once by hand so it learns the new mode. It never acts on chat from before you switched it on, and ignores a second finish within 20 seconds. The `/p list` reply appears in your chat when it asks. It needs the **Commands** permission (Settings > Mods), which the mod asks for by name: only `/q <mode>` and `/p list` are ever sent. In an Auto CS custom server it has nothing useful to do (that has its own replay menu), so leave it off there.

## Arrow counter

An arrow icon with the number of arrows you carry (inventory and offhand, tipped arrows included) on your HUD; the number turns red at 0. On the menu's **Arrow counter** page: switch it on or off, **Only with a bow** (shown only while a bow or crossbow is in your hand or offhand), **Size** with **-** / **+** (50%-400%; click the size to go back to 100%) and **Reset place and size** (beside the hotbar, 100%). **Move and resize on screen** hides the menu and frames the counter: drag it anywhere with the left mouse button, and drag any of the small squares on its corners to resize it, the opposite corner staying put (the size shows below it). Moving and resizing snap to the hotbar (its left and right edges, just beside it, its top, its middle height and the bottom of the screen), to the middle of the screen and to the screen edges; a purple line shows what it lined up with. Resizing until the top edge snaps to the hotbar's top makes the counter exactly as tall as the hotbar, and click **Done** (or right-click, Enter or Esc) when finished; **Reset** in the guide puts it back. Scrolling or **+** / **-** also resize it. The guide moves to the bottom of the screen while the counter is near the top. Its place is measured from the nearest screen edge or the middle (where the hotbar is), so it stays beside the hotbar when the window or GUI scale changes. It needs the **Player status** permission (Mods tab; to read the inventory) and **HUD**; without Player status it shows a yellow **?** instead of the number.

## CPS counter

A line of text such as `CPS: 7 | 3` (left | right mouse button clicks in the last second) on your HUD, top left by default. Open the **CPS counter** page from the start page: switch it on or off, **Right clicks** (show only the left button), **Size** with **-** / **+** (50%-400%; click the size to go back to 100%) and **Reset place and size**. **Move and resize on screen** (here or on the Arrow counter page) shows the arrow counter and the CPS readout together, each in its own frame, so you can place both in one go: drag either one anywhere, drag a corner square to resize, it snaps to the hotbar and the screen's middle and edges. Clicks count only in gameplay with the menu closed. Its place is measured from the nearest screen edge or the middle, so it stays there when the window or GUI scale changes. Needs the **Controls** and **HUD** permissions.

## FPS counter

`FPS: 144`, the frames per second (updated twice a second), on your HUD, below the CPS counter by default. Open the **FPS counter** page from the start page (or use its switch there): **Show FPS**, **Show coordinates** (off by default; see below), **Move and resize on screen**, **Size** with **-** / **+** (50%-400%; click the size to go back to 100%) and **Reset place and size**. In the move editor it has its own frame beside the arrow counter's and the CPS counter's, so it can sit anywhere. Needs the **HUD** permission.

**Coordinates:** with **Show coordinates** on, a second line under the FPS shows the block you stand in, `XYZ: 12 64 -30` (your feet's position rounded down, so it matches the game's own coordinates). It is part of the same element: it moves and resizes with it, and it can show on its own with **Show FPS** off, when it takes the top line. The position is read ten times a second and the line is sent to the screen only when a number changes, so it costs one small host call per tenth of a second and nothing while it is off or you are out of a world. Needs the **Player status** permission.

## Crosshair

Replaces the game's crosshair with your own. Open the **Crosshair** page from the start page and switch on **Show** (here, or with the switch beside the start page button; it is off until you do, so the game's crosshair stays until you choose otherwise). **Colour** at the top opens the same colour tools as the outline pages (bars, wheel, hex code, quick and saved colours), aimed at the crosshair's colour. **Style** steps through Cross, Cross + dot, T shape, X, Circle, Circle + dot, Dot and Custom (- and + go round; click the value for a cross again). **Draw your own** (beside **Colour**) opens a grid of pixels and switches the crosshair on in the Custom style: left click (or hold and drag) draws pixels, and right click (or hold and drag) erases them. The size beside **Colour** sets the grid to 16x16, 32x32 or 64x64 pixels (- and +; click it for 16x16); the drawing keeps its shape (each pixel becomes a square of pixels, or a pixel is drawn where any of its square was) and about its size on screen. **Mirror** (on by default) paints the same pixel in all four quarters, so a symmetric crosshair takes a quarter of the work; **Clear drawing** (click twice) empties the grid. Purple lines mark its middle, stronger lines every 4 pixels (every 8 at 64x64) help you count, and the pixel under the pointer is lit, with its mirrored pixels when Mirror is on. The drawing starts as a small cross centred on the grid. A drawing's pixels are sized in screen pixels, 1 to 8 (the thickness value is called **Pixel size** in the Custom style; Length and Gap are not used by a drawing). Rows of pixels are sent as rectangles, in steps of 16 a frame, at most 144 of them, and a drawing may have at most 320 runs of pixels in rows unlike the row above, since working those out and sending them costs a frame fuel: a pixel that would go past either is refused, and the line under the grid says so. **Colour** works for the drawing too, and **Reset crosshair** keeps the drawing. **Length**, **Thickness** and **Gap** (empty room in the middle, measured from the edge of the middle square) change in half GUI pixels, **Opacity** in steps of 10%, and **Outline** puts a thin dark edge round the shape so it shows on bright and dark ground (below 100% opacity it also darkens the shape's edges a little, as it is drawn under the shape); **Reset crosshair** (click twice) puts the look back and leaves it switched on. Every edge is placed on whole screen pixels, so thin lines stay sharp at any GUI scale; a circle's ring runs through the middle of where the arms would be. The game's crosshair is hidden only while yours is on screen, and yours follows the game's: it hides with **F1**, in third person and on the death screen. The page keeps the HUD visible, so you see every change as you make it. Needs the **HUD** permission.

## Zoom

Hold **Z** (changeable on the menu's **Zoom** page: click **Zoom key**, then press a key; **Escape** cancels; **Backspace** or **Delete** unbinds it, so the zoom does nothing until you pick a key again) and the view narrows smoothly; let go and it eases back. The Zoom page has: **Zoom** on/off; **Mode** (hold the key, or press it to switch the zoom on and off); **Scroll zoom** (it moves your hotbar slot; see below); **Relative sensitivity** (you look more slowly while zoomed, in proportion to the zoom); **Zoom level** (1.5x-10x, 4x by default), **Zoom time** (0 = instant to 1 s) and **Scroll step** (5%-50% each notch), each with **-** / **+** buttons (click the value itself to go back to its default); and **Reset zoom**.

- **Changing the zoom while zooming:** **=** zooms in and **-** zooms out, one step each press. They never touch your hotbar.
- **Scroll zoom (off by default):** with it switched on, scrolling the mouse wheel while the zoom key is held zooms in (wheel up) or out (wheel down). A mod cannot read or block the wheel during gameplay: the game moves your selected hotbar slot on every wheel turn, and the zoom only notices that. So with this on, **your hotbar slot moves while you scroll**, and no mod can stop that (it needs a change in Cinnabar itself). Switch it off to keep your slot still; **=** / **-** still work. Pressing a number key to pick a slot is not counted as a scroll.
- Each zoom starts from the **Zoom level** setting; scrolling only changes the zoom you are holding. The zoom needs the **Camera rotation** permission and works only in gameplay, with no menu open. It is a fresh implementation inspired by Zoomify by isXander, not a copy of its code. The camera API limits the zoom to 10x and cannot widen the view past normal.

## Hides with the HUD

Like Cinnabar's own selection, the outline disappears with **F1** (Hide HUD) and on the death screen.

## Mobs and players

While holding a bow or crossbow, player hitboxes also extend to 48 blocks when **All mobs** is on. Other mobs remain within 12 blocks; putting the bow away restores the normal player range. Visibility is still limited to the host's nearest actors.

Nearby arrow projectiles use the Mob hitbox switch and their own **Arrow** colour button on that page, even when **All mobs** is off. The outline follows the collision box reported by Cinnabar at the arrow tip. Up to eight arrows within 48 blocks are outlined, subject to the nearest-actor limit.

When one stands in front of the block, Cinnabar targets it instead, so its hitbox is outlined rather than the block behind it. Hitboxes are drawn with the **Mob hitbox** page's settings. With **All mobs: On** (the default) every mob within 12 blocks is outlined (the nearest 8 hitboxes; in first person, mobs wholly behind you are skipped, as they cannot be seen, so the 8 go to ones you can; only the players and mobs in range, and at most the 12 nearest, are read each frame, which only matters in a very dense crowd), and the one you aim at switches to the **Aimed colour**; with it off only the aimed mob is outlined, in the mob colours.

A mob's hitbox is drawn as its whole box, all twelve edges (the far ones show through, like Cinnabar's own hitbox view), as one object per mob: this is what keeps a crowd of outlined mobs cheap. Blocks still show only the edges facing you. Three of the twelve edges are traced twice, which only shows with a see-through line colour (they look slightly brighter).

After an Auto CS game ends, choose **Replay** on the game-over overview to open the map pool and create the next server. The map pool reopens after Hive moves you to another server. Remembered invites and spectators carry over until `/autocs end`.

On Red versus Blue maps, Auto CS sets non-party players to Blue and leaves party roles unchanged.
