// STRUDEL EDM BLUEPRINT PALATE
// 32 handcrafted, playable Strudel patterns.
// Paste any block into https://strudel.cc and hit Ctrl+Enter.
// Syntax follows official docs: s(), note(), .bank(), .gain(), .room(), .size(),
// .lpf(), .lpq(), .hpf(), .shape(), .delay(), .delaytime(), .delayfeedback(),
// .pan(), .phaser(), .clip(). Banks: RolandTR909, RolandTR808.
// setcps(x) where BPM ≈ x * 240 (one cycle = one 4/4 bar).

// =============================================================================
// 01  FOUR ON THE FLOOR — classic dark house
// =============================================================================
// [01] WAREHOUSE FLOOR
// Dark house, 128 BPM. Punchy 909, rolling sub, sparse stab.
setcps(0.53)
stack(
  s("bd*4").bank("RolandTR909").shape(0.3).gain(0.95),
  s("~ cp ~ cp").bank("RolandTR909").room(0.4).gain(0.8),
  s("hh*8").bank("RolandTR909").gain(0.45).pan(sine.range(0.3, 0.7).slow(4)),
  s("~ ~ ~ oh").bank("RolandTR909").gain(0.35).room(0.3),
  note("<a1 a1 c2 a1 f1 f1 g1 g1>*2")
    .s("sawtooth").lpf(280).lpq(2).shape(0.2).gain(0.75),
  note("<[a2,c3,e3] [c3,e3,g3] [f2,a2,c3] [g2,bb2,d3]>/2")
    .s("square").lpf(900).room(0.6).size(4).gain(0.28)
)

// =============================================================================
// 02  OFFBEAT HOUSE — open hat on the and
// =============================================================================
// [02] OFFBEAT PRESSURE
// House, 124 BPM. Open hat on the offbeat, chord stabs.
setcps(0.517)
stack(
  s("bd*4").bank("RolandTR909").gain(0.95),
  s("~ cp ~ cp").bank("RolandTR909").room(0.35).gain(0.75),
  s("~ oh ~ oh").bank("RolandTR909").gain(0.4).room(0.25),
  s("hh*8").bank("RolandTR808").gain(0.3),
  note("<d1!4 f1!4 c1!4 g1!4>")
    .s("sine").lpf(180).gain(0.8),
  note("<[d3,f3,a3] [f3,a3,c4] [c3,e3,g3] [g3,bb3,d4]>")
    .s("sawtooth").clip(0.15).lpf(1400).room(0.5).gain(0.32)
    .struct("~ x ~ x")
)

// =============================================================================
// 03  DEEP HOUSE — slower, swung hats, warm pad
// =============================================================================
// [03] DEEP ROOM
// Deep house, 122 BPM.
setcps(0.508)
stack(
  s("bd*4").bank("RolandTR909").shape(0.15).gain(0.9),
  s("~ cp ~ cp").bank("RolandTR808").room(0.55).size(5).gain(0.7),
  s("hh*8").bank("RolandTR808").gain("[0.25 0.4]*4").swing(0.08),
  note("<e1 ~ g1 ~ a1 ~ e1 d1>")
    .s("sine").gain(0.7),
  note("<[e2,g2,b2,d3] [a2,c3,e3,g3] [d2,f2,a2,c3] [g2,b2,d3,f3]>/2")
    .s("sawtooth").lpf(sine.range(400, 1400).slow(8)).lpq(1).room(0.7).size(6).gain(0.3)
)

// =============================================================================
// 04  PEAK-TIME MELODIC TECHNO
// =============================================================================
// [04] PEAK MELODIC
// Melodic techno, 132 BPM. Driving kick, long supersaw-style saw chords.
setcps(0.55)
stack(
  s("bd*4").bank("RolandTR909").shape(0.35).gain(0.98),
  s("~ cp ~ cp").bank("RolandTR909").room(0.5).gain(0.82),
  s("hh*16").bank("RolandTR909").gain(0.28),
  s("[~ rd]*2").bank("RolandTR909").gain(0.22).room(0.6),
  note("<d1!4 f1!4 bb0!4 c1!4>*2")
    .s("sawtooth").lpf(sine.range(200, 900).slow(6)).lpq(4).shape(0.3).gain(0.7),
  note("<[d3,f3,a3,c4] [bb2,d3,f3,a3] [g2,bb2,d3,f3] [a2,c3,e3,g3]>/2")
    .s("sawtooth").lpf(sine.range(800, 2800).slow(8)).room(0.75).size(6).phaser(2).gain(0.32),
  note("<a4 ~ d5 f5 ~ a5 ~ f5 d5>")
    .s("triangle").lpf(3500).room(0.8).delay(0.35).delaytime(0.25).delayfeedback(0.4).gain(0.4)
)

// =============================================================================
// 05  RAW WAREHOUSE TECHNO
// =============================================================================
// [05] RAW WAREHOUSE
// Hard techno, 140 BPM. Distorted kick, almost no melody.
setcps(0.583)
stack(
  s("bd*4").bank("RolandTR909").shape(0.55).gain(1),
  s("~ cp ~ [cp sd]").bank("RolandTR909").room(0.3).gain(0.8),
  s("hh*8").bank("RolandTR909").gain(0.4),
  s("[~ oh]*4").bank("RolandTR909").gain(0.3),
  note("c1*8").s("sawtooth").lpf(sine.range(120, 500).slow(4)).lpq(6).shape(0.4).gain(0.55),
  note("<c2 eb2 f2 g2>*4").s("square").lpf(600).hpf(200).gain(0.2).room(0.4)
)

// =============================================================================
// 06  MINIMAL TECHNO
// =============================================================================
// [06] MINIMAL LOOP
// Minimal techno, 128 BPM. Few notes, long filter move.
setcps(0.533)
stack(
  s("bd*4").bank("RolandTR909").gain(0.92),
  s("~ ~ cp ~").bank("RolandTR909").room(0.45).gain(0.6),
  s("hh*4").bank("RolandTR909").gain(0.35),
  note("a1*4").s("sawtooth")
    .lpf(sine.range(150, 1200).slow(16)).lpq(8).gain(0.6),
  note("a3 [~ c4] e4 ~").s("sine").room(0.7).delay(0.4).delayfeedback(0.5).gain(0.25)
)

// =============================================================================
// 07  DUB TECHNO
// =============================================================================
// [07] DUB CHORD WASH
// Dub techno, 118 BPM. Chords drowned in delay.
setcps(0.492)
stack(
  s("bd*4").bank("RolandTR808").shape(0.1).gain(0.85),
  s("~ cp ~ cp").bank("RolandTR808").room(0.6).gain(0.45),
  s("hh*8").bank("RolandTR808").gain(0.22),
  note("<d1 f1 c1 g1>")
    .s("sine").gain(0.65),
  note("<[d3,f3,a3] [f3,a3,c4] [c3,e3,g3] [bb2,d3,f3]>")
    .s("sawtooth").lpf(700).room(0.9).size(8)
    .delay(0.55).delaytime(0.375).delayfeedback(0.65).gain(0.28)
)

// =============================================================================
// 08  ACID HOUSE
// =============================================================================
// [08] 303 HOUSE
// Acid house, 126 BPM. Resonant saw bass doing the 303 job.
setcps(0.525)
stack(
  s("bd*4").bank("RolandTR909").gain(0.95),
  s("~ cp ~ cp").bank("RolandTR909").gain(0.75),
  s("hh*8").bank("RolandTR808").gain(0.4),
  s("~ oh ~ oh").bank("RolandTR909").gain(0.3),
  note("<c2 c2 eb2 c2 f2 c2 g2 bb1>*2")
    .s("sawtooth").clip(0.4)
    .lpf(sine.range(250, 3200).slow(4)).lpq(12).shape(0.35).gain(0.65)
)

// =============================================================================
// 09  ACID TECHNO
// =============================================================================
// [09] ACID RAVE
// Acid techno, 138 BPM.
setcps(0.575)
stack(
  s("bd*4").bank("RolandTR909").shape(0.45).gain(1),
  s("~ cp ~ cp").bank("RolandTR909").gain(0.8),
  s("hh*16").bank("RolandTR909").gain(0.3),
  note("[f1 f2 c2 gs1 f1 ds2 d2 c2]*2")
    .s("sawtooth").clip(0.55)
    .lpf(sine.range(300, 4000).slow(3)).lpq(14).shape(0.5).gain(0.6)
)

// =============================================================================
// 10  PROGRESSIVE HOUSE
// =============================================================================
// [10] PROGRESSIVE LIFT
// Progressive house, 126 BPM. Long chord, plucked arp.
setcps(0.525)
stack(
  s("bd*4").bank("RolandTR909").gain(0.92),
  s("~ cp ~ cp").bank("RolandTR909").room(0.4).gain(0.7),
  s("hh*8").bank("RolandTR909").gain(0.35),
  note("<a0!8 f0!8 g0!8 e0!8>")
    .s("sine").gain(0.7),
  note("<[a2,c3,e3,g3] [f2,a2,c3,e3] [g2,bb2,d3,f3] [e2,g2,b2,d3]>/4")
    .s("sawtooth").lpf(sine.range(600, 2200).slow(16)).room(0.8).size(6).gain(0.3),
  note("[a3 c4 e4 g4 e4 c4]*2")
    .s("triangle").lpf(3000).delay(0.3).delaytime(0.1875).gain(0.28)
)

// =============================================================================
// 11  TECH HOUSE
// =============================================================================
// [11] TECH HOUSE GROOVE
// Tech house, 126 BPM. Percussion heavy, short bass.
setcps(0.525)
stack(
  s("bd*4").bank("RolandTR909").gain(0.95),
  s("~ cp ~ cp").bank("RolandTR909").gain(0.85),
  s("[hh hh hh oh]*2").bank("RolandTR909").gain(0.4),
  s("~ perc ~ perc").bank("RolandTR808").gain(0.35).room(0.2),
  note("<a1 ~ a1 c2 ~ a1 g1 ~>*2")
    .s("square").lpf(250).clip(0.3).gain(0.7),
  note("e4 ~ ~ g4 ~ ~ a4 ~").s("sine").hpf(800).delay(0.25).gain(0.22)
)

// =============================================================================
// 12  BASS HOUSE
// =============================================================================
// [12] BASS HOUSE SLAP
// Bass house, 128 BPM. Octave bass, short chords.
setcps(0.533)
stack(
  s("bd*4").bank("RolandTR909").shape(0.4).gain(1),
  s("~ cp ~ cp").bank("RolandTR909").shape(0.2).gain(0.85),
  s("hh*8").bank("RolandTR808").gain(0.4),
  note("<[a1 a2]*4 [f1 f2]*4 [g1 g2]*4 [e1 e2]*4>")
    .s("sawtooth").lpf(sine.range(180, 1400).slow(2)).lpq(6).shape(0.45).gain(0.75),
  note("<[a3,c4,e4] [f3,a3,c4] [g3,bb3,d4] [e3,g3,b3]>")
    .s("square").clip(0.2).lpf(1600).gain(0.3)
)

// =============================================================================
// 13  MELODIC HOUSE
// =============================================================================
// [13] MELODIC HOUSE
// Melodic house, 122 BPM.
setcps(0.508)
stack(
  s("bd*4").bank("RolandTR909").gain(0.9),
  s("~ cp ~ cp").bank("RolandTR909").room(0.5).gain(0.7),
  s("hh*8").bank("RolandTR808").gain(0.32).pan(sine.slow(4)),
  note("<c1!4 eb1!4 ab0!4 bb0!4>")
    .s("sine").gain(0.72),
  note("<[c3,eb3,g3,bb3] [eb3,g3,bb3,d4] [ab2,c3,eb3,g3] [bb2,d3,f3,ab3]>/2")
    .s("sawtooth").lpf(sine.range(700, 2400).slow(8)).room(0.75).phaser(1).gain(0.32),
  note("g4 eb4 c4 ~ bb4 g4 eb4 d4")
    .s("triangle").room(0.85).delay(0.4).delayfeedback(0.45).gain(0.38)
)

// =============================================================================
// 14  DARK MELODIC TECHNO
// =============================================================================
// [14] DARK MELODIC
// Dark melodic techno, 136 BPM. Minor, tense.
setcps(0.567)
stack(
  s("bd*4").bank("RolandTR909").shape(0.4).gain(0.98),
  s("~ cp ~ cp").bank("RolandTR909").room(0.4).gain(0.8),
  s("hh*16").bank("RolandTR909").gain(0.26),
  note("<a0!4 f0!4 e0!4 g0!4>*2")
    .s("sawtooth").lpf(220).shape(0.35).gain(0.7),
  note("<[a2,c3,e3] [f2,a2,c3] [e2,g2,b2] [g2,bb2,d3]>/2")
    .s("sawtooth").lpf(sine.range(500, 1800).slow(8)).room(0.8).gain(0.3),
  note("e4 ~ g4 a4 c5 ~ a4 g4")
    .s("square").lpf(2800).delay(0.4).delaytime(0.1875).gain(0.28)
)

// =============================================================================
// 15  INDUSTRIAL TECHNO
// =============================================================================
// [15] INDUSTRIAL
// Industrial techno, 145 BPM.
setcps(0.604)
stack(
  s("bd*4").bank("RolandTR909").shape(0.65).gain(1),
  s("~ sd ~ sd").bank("RolandTR909").shape(0.4).room(0.25).gain(0.85),
  s("hh*8").bank("RolandTR909").gain(0.45).hpf(4000),
  s("lt*2").bank("RolandTR909").lpf(400).gain(0.4),
  note("<c1 c1 gs0 gs0 f0 f0 g0 g0>*2")
    .s("square").lpf(180).shape(0.6).gain(0.65),
  note("c3 ~ eb3 ~ fs3 ~ g3 ~").s("sawtooth").lpf(500).shape(0.5).gain(0.25)
)

// =============================================================================
// 16  HARD TECHNO
// =============================================================================
// [16] HARD TECHNO
// Hard techno, 150 BPM.
setcps(0.625)
stack(
  s("bd*4").bank("RolandTR909").shape(0.6).gain(1),
  s("~ cp ~ cp").bank("RolandTR909").shape(0.3).gain(0.9),
  s("[hh*2 oh]*4").bank("RolandTR909").gain(0.4),
  note("f1*8").s("sawtooth").lpf(sine.range(100, 700).fast(2)).lpq(8).shape(0.55).gain(0.6),
  note("<f3 ab3 c4 eb4>*4").s("square").hpf(400).lpf(2000).gain(0.22)
)

// =============================================================================
// 17  TRANCE GATE
// =============================================================================
// [17] TRANCE GATE
// Trance, 138 BPM. Gated chord, rolling bass.
setcps(0.575)
stack(
  s("bd*4").bank("RolandTR909").gain(0.95),
  s("~ cp ~ cp").bank("RolandTR909").room(0.45).gain(0.75),
  s("hh*16").bank("RolandTR909").gain(0.28),
  s("~ oh ~ oh").bank("RolandTR909").gain(0.3),
  note("<a1!8 g1!8 f1!8 e1!8>*2")
    .s("sawtooth").lpf(400).gain(0.65),
  note("<[a3,c4,e4] [g3,b3,d4] [f3,a3,c4] [e3,g3,b3]>")
    .s("sawtooth").struct("x*8").clip(0.2)
    .lpf(sine.range(800, 4000).slow(8)).room(0.6).gain(0.3),
  note("e5 a4 c5 e5 a5 e5 c5 a4")
    .s("triangle").delay(0.35).delayfeedback(0.4).gain(0.32)
)

// =============================================================================
// 18  UPLIFTING TRANCE LEAD
// =============================================================================
// [18] UPLIFT LEAD
// Uplifting trance, 140 BPM. Major-feeling lift over minor bass.
setcps(0.583)
stack(
  s("bd*4").bank("RolandTR909").gain(0.95),
  s("~ cp ~ cp").bank("RolandTR909").room(0.5).gain(0.75),
  s("hh*16").bank("RolandTR909").gain(0.25),
  note("<a1 c2 g1 f1>*4").s("sawtooth").lpf(350).gain(0.6),
  note("<[a3,c4,e4,g4] [c4,e4,g4,b4] [g3,b3,d4,f4] [f3,a3,c4,e4]>/2")
    .s("sawtooth").lpf(2200).room(0.7).phaser(3).gain(0.28),
  note("<e5 g5 a5 c6 a5 g5 e5 c5>*2")
    .s("sawtooth").lpf(4500).delay(0.4).delaytime(0.1875).delayfeedback(0.45).gain(0.35)
)

// =============================================================================
// 19  PSYTRANCE ROLL
// =============================================================================
// [19] PSY ROLL
// Psytrance, 145 BPM. 16th rolling bass.
setcps(0.604)
stack(
  s("bd*4").bank("RolandTR909").gain(0.95),
  s("~ cp ~ cp").bank("RolandTR909").gain(0.7),
  s("hh*16").bank("RolandTR909").gain("[0.2 0.45]*8"),
  note("<e1 e1 e1 g1 e1 e1 d1 e1>*8")
    .s("sawtooth").lpf(sine.range(200, 1800).fast(2)).lpq(8).gain(0.55),
  note("<b4 e5 g5 b5 g5 e5>*2")
    .s("square").hpf(1000).delay(0.25).gain(0.22)
)

// =============================================================================
// 20  DRUM AND BASS ROLLER
// =============================================================================
// [20] DNB ROLLER
// Drum and bass, 174 BPM. Two-step kick, rolling hats, Reese.
setcps(0.725)
stack(
  s("bd ~ ~ [bd ~ bd ~]").bank("RolandTR909").shape(0.3).gain(0.95),
  s("~ ~ sd ~").bank("RolandTR909").room(0.35).gain(0.9),
  s("hh*16").bank("RolandTR808").gain(perlin.range(0.2, 0.55)),
  note("<e1!4 c1!4 a0!4 b0!4>*4")
    .s("sawtooth").lpf(sine.range(180, 700).slow(4)).lpq(4).shape(0.4).gain(0.7),
  note("<[e3,g3,b3] [c3,e3,g3] [a2,c3,e3] [b2,d3,fs3]>/2")
    .s("sawtooth").lpf(1600).room(0.7).gain(0.25)
)

// =============================================================================
// 21  LIQUID DNB
// =============================================================================
// [21] LIQUID
// Liquid drum and bass, 170 BPM. Softer, melodic.
setcps(0.708)
stack(
  s("bd ~ ~ bd ~ ~ bd ~").bank("RolandTR808").gain(0.85),
  s("~ ~ sd ~").bank("RolandTR909").room(0.55).gain(0.75),
  s("hh*8").bank("RolandTR808").gain(0.3).pan(sine.slow(3)),
  note("<a1 ~ e1 ~ f1 ~ g1 ~>")
    .s("sine").gain(0.65),
  note("c5 e5 g5 a5 g5 e5 c5 a4")
    .s("triangle").room(0.85).delay(0.35).delaytime(0.15).gain(0.35)
)

// =============================================================================
// 22  NEURO BASS
// =============================================================================
// [22] NEURO
// Neurofunk-leaning DnB, 174 BPM. Moving filter bass.
setcps(0.725)
stack(
  s("bd ~ ~ [~ bd] ~ ~ bd ~").bank("RolandTR909").shape(0.4).gain(0.95),
  s("~ ~ sd ~").bank("RolandTR909").shape(0.25).gain(0.9),
  s("hh*16").bank("RolandTR909").gain(0.3),
  note("<f1 fs1 g1 a1 g1 f1 e1 f1>*8")
    .s("sawtooth").lpf(sine.range(150, 2200).fast(4)).lpq(10).shape(0.55).gain(0.6)
)

// =============================================================================
// 23  WAVE PHONK DRUMS
// =============================================================================
// [23] PHONK DRIFT
// Wave phonk, ~140 BPM. Cowbell, 808 clap, distorted kick.
setcps(0.583)
stack(
  s("bd*<2 4 4 [4 8]>").bank("RolandTR909").shape(0.4).gain(0.95),
  s("~ [sd, cp] ~ [sd, cp]").bank("RolandTR808").room(0.4).gain(0.85),
  s("hh*8").bank("RolandTR808").gain(0.4),
  s("<~ [~ cb*2] ~ [cb ~ cb*2]>").bank("RolandTR808").room(0.5).delay(0.25).gain(0.5),
  note("<d1!6 f1!2 bb0!6 c1!2>*4")
    .s("sawtooth").lpf(400).shape(0.45).gain(0.7)
)

// =============================================================================
// 24  DRIFT PHONK
// =============================================================================
// [24] DRIFT
// Drift phonk, 150 BPM. Cowbell melody on top of a heavy kick.
setcps(0.625)
stack(
  s("bd*4").bank("RolandTR909").shape(0.5).gain(1),
  s("~ cp ~ cp").bank("RolandTR808").gain(0.8),
  s("hh*8").bank("RolandTR808").gain(0.35),
  note("<c5 eb5 g5 eb5 c5 g5 f5 eb5>*2")
    .s("cb").bank("RolandTR808").room(0.5).delay(0.3).gain(0.55),
  note("<c1 ab0 f0 g0>*4").s("sawtooth").lpf(250).shape(0.5).gain(0.75)
)

// =============================================================================
// 25  RETROWAVE
// =============================================================================
// [25] RETROWAVE
// Retrowave / outrun, 100 BPM. Slow, wide chords, simple lead.
setcps(0.417)
stack(
  s("bd ~ bd ~").bank("RolandTR808").gain(0.85),
  s("~ sd ~ sd").bank("RolandTR808").room(0.6).gain(0.75),
  s("hh*8").bank("RolandTR808").gain(0.28),
  note("<a1!4 f1!4 g1!4 e1!4>")
    .s("sawtooth").lpf(300).gain(0.65),
  note("<[a2,c3,e3,g3] [f2,a2,c3,e3] [g2,b2,d3,f3] [e2,g2,b2,d3]>/2")
    .s("sawtooth").lpf(sine.range(600, 2400).slow(8)).room(0.8).size(6).phaser(2).gain(0.35),
  note("e4 g4 a4 c5 a4 g4 e4 c4")
    .s("square").lpf(2500).delay(0.45).delayfeedback(0.4).gain(0.3)
)

// =============================================================================
// 26  SYNTHWAVE NIGHT DRIVE
// =============================================================================
// [26] NIGHT DRIVE
// Synthwave, 110 BPM.
setcps(0.458)
stack(
  s("bd sd bd sd").bank("RolandTR808").room(0.3).gain(0.85),
  s("hh*8").bank("RolandTR808").gain(0.25),
  note("<c2!4 ab1!4 bb1!4 g1!4>")
    .s("sawtooth").lpf(280).gain(0.65),
  note("<[c3,eb3,g3,bb3] [ab2,c3,eb3,g3] [bb2,d3,f3,ab3] [g2,bb2,d3,f3]>/2")
    .s("sawtooth").lpf(1800).room(0.75).phaser(3).gain(0.32),
  note("g4 bb4 c5 eb5 c5 bb4 g4 eb4")
    .s("triangle").delay(0.4).room(0.8).gain(0.35)
)

// =============================================================================
// 27  FUTURE BASS DROP
// =============================================================================
// [27] FUTURE BASS
// Future bass, 150 BPM (half-time feel). Supersaw chords, trap hats.
setcps(0.625)
stack(
  s("bd ~ ~ bd ~ ~ [bd bd] ~").bank("RolandTR808").shape(0.35).gain(0.95),
  s("~ ~ sd ~").bank("RolandTR808").room(0.5).gain(0.85),
  s("hh*8").bank("RolandTR808").gain("[0.2 0.5 0.3 0.6]*2"),
  s("[hh*16]").bank("RolandTR808").mask("<0 0 1 0>").gain(0.35),
  note("<g1!4 e1!4 c1!4 d1!4>")
    .s("sawtooth").lpf(200).shape(0.3).gain(0.7),
  note("<[g3,b3,d4,fs4] [e3,g3,b3,d4] [c3,e3,g3,b3] [d3,fs3,a3,c4]>")
    .s("sawtooth").lpf(sine.range(800, 3500).slow(4)).room(0.6).gain(0.35)
)

// =============================================================================
// 28  MELODIC DUBSTEP
// =============================================================================
// [28] MELODIC DUB
// Melodic dubstep, 140 BPM half-time.
setcps(0.583)
stack(
  s("bd ~ ~ ~ bd ~ ~ bd").bank("RolandTR909").shape(0.4).gain(1),
  s("~ ~ sd ~").bank("RolandTR909").room(0.45).gain(0.85),
  s("hh*8").bank("RolandTR808").gain(0.3),
  note("<d1 ~ ~ f1 ~ ~ c1 ~>")
    .s("sawtooth").lpf(sine.range(120, 800).slow(2)).shape(0.5).gain(0.7),
  note("<[d3,f3,a3,c4] [f3,a3,c4,e4] [c3,e3,g3,b3] [a2,c3,e3,g3]>/2")
    .s("sawtooth").lpf(2000).room(0.75).delay(0.3).gain(0.3),
  note("a4 f4 d4 ~ c5 a4 f4 d4")
    .s("triangle").room(0.85).gain(0.32)
)

// =============================================================================
// 29  UK GARAGE 2-STEP
// =============================================================================
// [29] UKG
// UK garage, 132 BPM. Shuffled 2-step, swung hats.
setcps(0.55)
stack(
  s("bd ~ ~ bd ~ bd ~ ~").bank("RolandTR909").gain(0.9),
  s("~ cp ~ ~ cp ~ ~ cp").bank("RolandTR909").room(0.35).gain(0.7),
  s("hh*8").bank("RolandTR909").swing(0.12).gain(0.35),
  note("<a1 ~ c2 ~ g1 ~ a1 f1>")
    .s("sine").gain(0.7),
  note("<[a3,c4,e4] [c4,e4,g4] [g3,bb3,d4] [f3,a3,c4]>")
    .s("sawtooth").clip(0.25).lpf(1800).room(0.4).gain(0.28)
)

// =============================================================================
// 30  BREAKBEAT
// =============================================================================
// [30] BREAKS
// Breakbeat, 135 BPM.
setcps(0.562)
stack(
  s("bd ~ ~ bd sd ~ bd ~, ~ ~ sd ~ ~ sd ~ sd").bank("RolandTR909").gain(0.85),
  s("hh*8").bank("RolandTR909").gain(0.35),
  s("~ oh ~ ~ ~ oh ~ oh").bank("RolandTR909").gain(0.3),
  note("<e1!4 g1!4 a1!4 d1!4>*2")
    .s("sawtooth").lpf(sine.range(200, 900).slow(4)).gain(0.65),
  note("<[e3,g3,b3] [g3,b3,d4] [a3,c4,e4] [d3,f3,a3]>/2")
    .s("square").lpf(1500).room(0.5).gain(0.25)
)

// =============================================================================
// 31  ELECTRO
// =============================================================================
// [31] ELECTRO
// Electro, 128 BPM. Dry 808, robotic bass.
setcps(0.533)
stack(
  s("bd ~ ~ bd ~ bd ~ ~").bank("RolandTR808").gain(0.95),
  s("~ sd ~ ~ sd ~ sd ~").bank("RolandTR808").gain(0.8),
  s("hh*8").bank("RolandTR808").gain(0.3),
  s("cb ~ cb cb ~ cb ~ cb").bank("RolandTR808").gain(0.35),
  note("<c2 c2 eb2 c2 g1 g1 bb1 g1>*2")
    .s("square").lpf(500).clip(0.3).gain(0.6)
)

// =============================================================================
// 32  AMBIENT TECHNO BED
// =============================================================================
// [32] AMBIENT BED
// Ambient techno, 110 BPM. Soft kick, long pad, almost no hats.
setcps(0.458)
stack(
  s("bd ~ ~ ~").bank("RolandTR808").gain(0.6).room(0.4),
  s("~ ~ ~ cp").bank("RolandTR808").room(0.7).gain(0.35),
  note("<d2 a2 f2 c2>/4")
    .s("sine").gain(0.4).room(0.8),
  note("<[d3,f3,a3,c4] [a2,c3,e3,g3] [f2,a2,c3,e3] [c3,e3,g3,b3]>/4")
    .s("sawtooth").lpf(sine.range(300, 1200).slow(16)).room(0.9).size(8).gain(0.28),
  note("a4 ~ ~ f4 ~ ~ d4 ~")
    .s("triangle").delay(0.5).delayfeedback(0.6).room(0.85).gain(0.22)
)