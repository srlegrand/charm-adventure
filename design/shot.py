# Reference screenshot (image only): target look for environment and character finish.
import math, random
K = '#0b0706'  # outline black
SW = 8
def S(w=SW): return f'stroke="{K}" stroke-width="{w}" stroke-linejoin="round" stroke-linecap="round"'
def d(a, l): r = math.radians(a); return (math.sin(r) * l, math.cos(r) * l)
def add(p, q): return (p[0] + q[0], p[1] + q[1])
def tube(p, q, w0, w1, fill):
    dx, dy = q[0] - p[0], q[1] - p[1]; L = math.hypot(dx, dy) or 1; nx, ny = -dy / L, dx / L
    quad = f'M{p[0]+nx*w0:.1f},{p[1]+ny*w0:.1f} L{q[0]+nx*w1:.1f},{q[1]+ny*w1:.1f} L{q[0]-nx*w1:.1f},{q[1]-ny*w1:.1f} L{p[0]-nx*w0:.1f},{p[1]-ny*w0:.1f}Z'
    caps = f'<circle cx="{p[0]:.1f}" cy="{p[1]:.1f}" r="{w0}" F/><circle cx="{q[0]:.1f}" cy="{q[1]:.1f}" r="{w1}" F/><path d="{quad}" F/>'
    return caps.replace('F', f'fill="{fill}" {S(SW*2)}') + caps.replace('F', f'fill="{fill}"')
def g(x, y, b, s=1.0, rot=0, flip=False): return f'<g transform="translate({x},{y}) rotate({rot}) scale({-s if flip else s},{s})">{b}</g>'

def head(c, r, skin, tilt, extra='', behind=''):
    face = (f'M{-.9*r},{.2*r} C{-1.1*r},{-.9*r} {.2*r},{-1.3*r} {.75*r},{-.7*r} L{.86*r},{-.3*r} L{1.12*r},{.06*r} L{.9*r},{.2*r} L{.96*r},{.4*r} L{.82*r},{.5*r} '
            f'L{.8*r},{.86*r} C{.4*r},{1.08*r} {0},{.9*r} {-.2*r},{.55*r} C{-.6*r},{.6*r} {-.85*r},{.45*r} {-.9*r},{.2*r}Z')
    return (f'<g transform="translate({c[0]:.1f},{c[1]:.1f}) rotate({tilt})">{behind}<path d="{face}" fill="{skin}" {S()}/>'
            f'<path d="M{.18*r},{-.44*r} L{.92*r},{-.18*r}" stroke="{K}" stroke-width="10" stroke-linecap="round"/>'
            f'<path d="M{.4*r},{-.2*r} L{.8*r},{-.08*r} L{.48*r},{0}Z" fill="{K}"/>'
            f'<path d="M{.6*r},{.52*r} L{.86*r},{.46*r}" stroke="{K}" stroke-width="5" stroke-linecap="round"/>{extra}</g>')

def human(pelvis, spine, legs, arms, look, weapon, hand='f', head_tilt=0):
    tf, sf, tb, sb = legs; uf, lf, ub, lb = arms
    pel = pelvis
    kf = add(pel, d(tf, 95)); ff = add(kf, d(sf, 100)); kb = add(pel, d(tb, 95)); fb = add(kb, d(sb, 100))
    neck = add(pel, d(180 - spine, 140)); hd = add(neck, d(180 - spine - head_tilt * .5, 46)); hd = (hd[0] + 10, hd[1])
    sh = add(pel, d(180 - spine, 124))
    ef = add(sh, d(uf, 76)); hf = add(ef, d(lf, 72)); eb = add(sh, d(ub, 76)); hb = add(eb, d(lb, 72))
    sk, top, leg = look['skin'], look['top'], look['leg']
    wsvg = lambda h: f'<g transform="translate({h[0]:.1f},{h[1]:.1f}) rotate({weapon[1]})">{weapon[0]}</g>'
    fist = lambda h: f'<circle cx="{h[0]:.1f}" cy="{h[1]:.1f}" r="15" fill="{sk}" {S()}/>'
    foot = lambda p, a: f'<g transform="translate({p[0]:.1f},{p[1]:.1f}) rotate({a})">{look["foot"]}</g>'
    o = ''
    if hand == 'b': o += wsvg(hb)
    o += tube(sh, eb, 14, 12, look.get('sleeve', sk)) + tube(eb, hb, 12, 10, sk) + fist(hb)
    o += tube(pel, kb, 23, 17, leg) + tube(kb, fb, 17, 12, leg) + foot(fb, -sb * .6)
    o += tube(pel, neck, 30, 30, top)
    if 'skirt' in look:
        a = (tf + tb) / 2; h1 = add(pel, d(a - 34, 92)); h2 = add(pel, d(a, 74)); h3 = add(pel, d(a + 34, 92)); w = add(pel, d(180 - spine, 30))
        o += f'<path d="M{w[0]-32:.1f},{w[1]:.1f} L{w[0]+32:.1f},{w[1]:.1f} L{h3[0]:.1f},{h3[1]:.1f} L{h2[0]:.1f},{h2[1]:.1f} L{h1[0]:.1f},{h1[1]:.1f}Z" fill="{top}" {S()}/>'
    o += look.get('torso_extra', lambda pel, neck: '')(pel, neck)
    o += tube(pel, kf, 24, 18, leg) + tube(kf, ff, 18, 12, leg) + foot(ff, -sf * .6)
    o += tube(neck, add(neck, d(180 - spine, 20)), 12, 11, sk)
    o += head(hd, 42, sk, head_tilt + spine * .3, look.get('face', ''), look.get('hair', ''))
    o += tube(sh, ef, 15, 12, look.get('sleeve', sk)) + tube(ef, hf, 12, 10, sk)
    if hand == 'f': o += wsvg(hf)
    o += fist(hf)
    return o

CROWBAR = f'<path d="M-8,60 L8,60 L8,-190 q4,-34 34,-30 l-2,16 q-18,-2 -18,18 L8,-170 L-8,-170Z" fill="#e2452e" {S()}/><path d="M0,50 L0,-180" stroke="#ff9a7a" stroke-width="3"/>'
SWORD = (f'<path d="M-10,-26 L-14,-290 L0,-330 L14,-290 L10,-26Z" fill="#f4fbff" {S()}/><path d="M0,-34 L0,-316" stroke="#9fd0ea" stroke-width="3"/>'
         f'<path d="M-42,-30 q42,-20 84,0 q-42,16 -84,0Z" fill="#f2b24a" {S()}/><path d="M-7,-22 h14 v48 h-14Z" fill="#2a0f2c" {S(5)}/><circle cx="0" cy="38" r="12" fill="#f2b24a" {S()}/>')
SHOE = f'<path d="M-16,-12 C-18,8 -14,14 0,14 L48,14 C58,12 54,-2 40,-4 L10,-14Z" fill="#1c1420" {S()}/><path d="M-12,9 L50,9" stroke="#f6efe0" stroke-width="5"/>'
POINT = f'<path d="M-10,-10 L10,-10 L16,8 L44,16 L-8,16Z" fill="#1c1420" {S()}/>'
def stripes(pel, neck):
    o = ''
    for i in range(1, 7):
        t = i / 7; x = pel[0] + (neck[0] - pel[0]) * t; y = pel[1] + (neck[1] - pel[1]) * t
        dx, dy = neck[0] - pel[0], neck[1] - pel[1]; L = math.hypot(dx, dy); nx, ny = -dy / L * 26, dx / L * 26
        o += f'<path d="M{x-nx:.1f},{y-ny:.1f} L{x+nx:.1f},{y+ny:.1f}" stroke="#27305c" stroke-width="9"/>'
    return o
SIMON = dict(skin='#f6dcc0', top='#f6efe0', leg='#3f4a72', foot=SHOE, sleeve='#f6efe0', torso_extra=stripes,
             face='<path d="M22,18 L33,36 C20,46 6,40 2,28Z" fill="#3a2f40"/><ellipse cx="-12" cy="2" rx="8" ry="12" fill="#e2b894" stroke="#0b0706" stroke-width="5"/><circle cx="-14" cy="18" r="7" fill="none" stroke="#f2b24a" stroke-width="4"/>')
R0 = random.Random(5); _p = []
for i in range(15):
    a = math.radians(150 + i * 15); r = 44 * (1.9 if i % 2 == 0 else 1.0) * R0.uniform(.9, 1.3)
    _p.append(f'{-math.sin(a)*r-10:.1f},{-math.cos(a)*r+6:.1f}')
CHARM = dict(skin='#b9774a', top='#b3262a', leg='#1c1420', foot=POINT, skirt=True,
             hair=f'<path d="M{" L".join(_p)}Z" fill="#241020" {S()}/>',
             face=f'<path d="M-44,-4 C-46,-56 30,-64 38,-28 C14,-40 -6,-30 -16,-4 L-22,30 L-40,24Z" fill="#241020" {S(5)}/><path d="M-6,-50 C-30,-30 -40,0 -62,22" stroke="#d9c08a" stroke-width="7" fill="none" stroke-linecap="round"/>')

def tomato(r, mouth=True, col='#d8392f', sh='#a3202a', legs=(20, -25), seed=1):
    o = ''
    for i, a in enumerate(legs):
        hip = (-r * .35 + i * r * .7, r * .8); kn = add(hip, d(a, r * .45))
        o += f'<path d="M{hip[0]:.1f},{hip[1]:.1f} L{kn[0]:.1f},{kn[1]:.1f}" stroke="{K}" stroke-width="16" stroke-linecap="round"/><path d="M{kn[0]-16:.1f},{kn[1]-6:.1f} h-26 q-12,8 0,14 h32Z" fill="#1c1420" {S(6)}/>'
    o += f'<path d="M-6,{-r*.92} C-8,{-r*1.3} 6,{-r*1.5} {r*.34},{-r*1.44} C{r*.5},{-r*1.38} {r*.44},{-r*1.2} {r*.3},{-r*1.24} C{r*.14},{-r*1.26} {r*.12},{-r*1.1} {r*.14},{-r*.92}Z" fill="#4c8a3c" {S()}/>'
    o += f'<path d="M{-r},0 C{-r*1.04},{-r*.6} {-r*.5},{-r*1.02} 0,{-r} C{r*.6},{-r} {r*1.02},{-r*.56} {r},0 C{r*1.02},{r*.6} {r*.56},{r} 0,{r} C{-r*.46},{r*1.02} {-r*.8},{r*.8} {-r*.88},{r*.46} C{-r*1.02},{r*.4} {-r*1.06},{r*.2} {-r},0Z" fill="{col}" {S()}/>'
    o += f'<path d="M{r},0 C{r*1.02},{r*.6} {r*.56},{r} 0,{r} C{-r*.2},{r} {-r*.4},{r*.96} {-r*.56},{r*.86} C0,{r*.86} {r*.64},{r*.54} {r*.7},{-r*.54} C{r*.9},{-r*.38} {r},{-r*.2} {r},0Z" fill="{sh}"/>'
    o += f'<path d="M{-r*.2},{-r*.98} l{-r*.3},{r*.06} l{r*.2},{r*.08} l{-r*.14},{r*.16} l{r*.3},{-r*.1} l{r*.1},{r*.2} l{r*.1},{-r*.2} l{r*.3},{r*.06} l{-r*.18},{-r*.16} l{r*.2},{-r*.1}Z" fill="#4c8a3c" {S(6)}/>'
    o += f'<path d="M{-r*.9},{-r*.4} C{-r*.74},{-r*.58} {-r*.56},{-r*.54} {-r*.42},{-r*.34} L{-r*.46},{-r*.24} C{-r*.6},{-r*.36} {-r*.74},{-r*.36} {-r*.88},{-r*.28}Z M{-r*.24},{-r*.36} C0,{-r*.58} {r*.26},{-r*.54} {r*.44},{-r*.38} L{r*.4},{-r*.26} C{r*.22},{-r*.38} {r*.02},{-r*.36} {-r*.16},{-r*.24}Z" fill="{K}"/>'
    o += f'<path d="M{-r*.82},{-r*.26} C{-r*.72},{-r*.34} {-r*.58},{-r*.32} {-r*.48},{-r*.22} C{-r*.6},{-r*.14} {-r*.74},{-r*.16} {-r*.82},{-r*.26}Z M{-r*.16},{-r*.22} C{r*.02},{-r*.34} {r*.22},{-r*.32} {r*.36},{-r*.22} C{r*.2},{-r*.08} 0,{-r*.08} {-r*.16},{-r*.22}Z" fill="#fff6e0" {S(5)}/><circle cx="{-r*.66}" cy="{-r*.23}" r="{r*.05}" fill="{K}"/><circle cx="{r*.06}" cy="{-r*.2}" r="{r*.065}" fill="{K}"/>'
    o += f'<path d="M{-r},{-r*.04} C{-r*1.14},{-r*.02} {-r*1.14},{r*.16} {-r},{r*.18}" fill="{col}" {S(6)}/>'
    if mouth:
        o += f'<path d="M{-r*.9},{r*.44} C{-r*.74},{r*.2} {-r*.26},{r*.12} {r*.2},{r*.26} C{r*.36},{r*.32} {r*.36},{r*.5} {r*.2},{r*.58} C{-r*.1},{r*.7} {-r*.5},{r*.78} {-r*.76},{r*.7}Z" fill="#2a0c10" {S()}/>'
        o += f'<path d="M{-r*.8},{r*.36} l{r*.1},{r*.14} l{r*.1},{-r*.18} l{r*.1},{r*.16} l{r*.1},{-r*.18} l{r*.1},{r*.16} l{r*.1},{-r*.16} l{r*.1},{r*.14} l{r*.1},{-r*.12} l{r*.1},{r*.12}" fill="#fff6e0" {S(5)}/>'
    else:
        o += f'<path d="M{-r*.84},{r*.5} C{-r*.5},{r*.3} {-r*.1},{r*.3} {r*.16},{r*.4}" fill="none" {S()}/>'
    return o

def burst(R):
    o = '<circle r="150" fill="#ffd9a0" opacity=".35" filter="url(#soft)"/>'
    pts = []
    for i in range(24):
        a = 2 * math.pi * i / 24; r = (150 if i % 2 == 0 else 62) * R.uniform(.8, 1.2); pts.append(f'{math.cos(a)*r:.0f},{math.sin(a)*r:.0f}')
    o += f'<path d="M{" L".join(pts)}Z" fill="#fff6e0" stroke="#f2a03c" stroke-width="6" stroke-linejoin="round"/>'
    for i in range(6):   # chunks of tomato thrown outward
        a = 2 * math.pi * i / 6 + .4; x, y = math.cos(a) * 120, math.sin(a) * 110; rot = math.degrees(a) + 90
        o += f'<g transform="translate({x:.0f},{y:.0f}) rotate({rot:.0f})"><path d="M-34,10 Q0,-52 34,10 Q0,26 -34,10Z" fill="#d8392f" {S()}/><path d="M-22,10 Q0,-22 22,10 Q0,18 -22,10Z" fill="#f58a6a"/><ellipse cx="-8" cy="4" rx="4" ry="6" fill="#fff6e0"/><ellipse cx="9" cy="5" rx="4" ry="6" fill="#fff6e0"/></g>'
    for i in range(26):
        a = R.uniform(0, 6.28); r = R.uniform(90, 250); s = R.uniform(5, 14)
        o += f'<ellipse transform="translate({math.cos(a)*r:.0f},{math.sin(a)*r:.0f}) rotate({math.degrees(a):.0f})" rx="{s*1.8:.0f}" ry="{s:.0f}" fill="#d8392f" {S(5)}/>'
    for i in range(10):
        a = 2 * math.pi * i / 10 + .2; o += f'<path d="M{math.cos(a)*170:.0f},{math.sin(a)*170:.0f} L{math.cos(a)*260:.0f},{math.sin(a)*260:.0f}" stroke="#fff6e0" stroke-width="6" stroke-linecap="round" opacity=".9"/>'
    o += f'<g transform="translate(70,-190) rotate(40)"><path d="M0,0 l-36,8 l24,10 l-16,20 l36,-12 l12,24 l12,-24 l36,8 l-22,-20 l24,-12Z" fill="#4c8a3c" {S(6)}/></g>'
    o += f'<g transform="translate(-150,-120)"><circle r="22" fill="#fff6e0" {S(6)}/><circle cx="5" cy="3" r="8" fill="{K}"/></g>'
    return o

def louis():
    f = '#f0d3a2'; dd = '#c79c63'
    return (f'<path d="M-150,-96 C-200,-130 -250,-92 -292,-124" stroke="{K}" stroke-width="20" fill="none" stroke-linecap="round"/><path d="M-150,-96 C-200,-130 -250,-92 -292,-124" stroke="{f}" stroke-width="7" fill="none" stroke-linecap="round"/>'
            + tube((-130, -84), (-200, -40), 16, 10, dd) + tube((-200, -40), (-280, -52), 10, 7, dd) + tube((80, -70), (150, -30), 13, 9, dd) + tube((150, -30), (222, -44), 9, 7, dd)
            + f'<path d="M-164,-92 C-110,-136 -30,-120 40,-118 C86,-126 122,-102 128,-74 C98,-46 60,-54 26,-56 C-20,-30 -80,-44 -126,-58 C-152,-62 -168,-76 -164,-92Z" fill="{f}" {S()}/>'
            + tube((-140, -76), (-204, -24), 17, 10, f) + tube((-204, -24), (-286, -8), 10, 7, f) + tube((90, -64), (176, -52), 13, 9, f) + tube((176, -52), (252, -74), 9, 7, f)
            + f'<path d="M92,-112 L150,-156 L190,-156 L178,-122 L128,-72Z" fill="{f}" {S()}/><path d="M132,-146 L172,-130 L124,-98 C96,-112 76,-130 24,-156 C70,-144 100,-144 132,-146Z" fill="#c8322c" {S(6)}/>'
            + f'<path d="M150,-162 L206,-182 L292,-152 L298,-140 L286,-132 L222,-128 L172,-124Z" fill="{f}" {S()}/><path d="M176,-172 L104,-196 L160,-148Z" fill="{dd}" {S()}/><circle cx="292" cy="-146" r="7" fill="{K}"/>'
            + f'<path d="M196,-168 L238,-158" stroke="{K}" stroke-width="7" stroke-linecap="round"/><path d="M212,-156 l22,4 l-18,6Z" fill="{K}"/><path d="M236,-128 L290,-118 L240,-112Z" fill="#2a0c10" {S(5)}/>')

W, H, GY = 1600, 900, 716
R = random.Random(21)
defs = f'''<defs>
<radialGradient id="bg" cx=".5" cy=".42" r=".75"><stop offset="0" stop-color="#6b4a26"/><stop offset=".35" stop-color="#3a2817"/><stop offset=".8" stop-color="#150e0a"/><stop offset="1" stop-color="#0a0706"/></radialGradient>
<linearGradient id="shaft" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#ffdf9a" stop-opacity=".34"/><stop offset="1" stop-color="#ffdf9a" stop-opacity="0"/></linearGradient>
<radialGradient id="pot" cx=".35" cy=".3" r=".8"><stop offset="0" stop-color="#8a5a2c"/><stop offset=".5" stop-color="#3a2414"/><stop offset="1" stop-color="#120b08"/></radialGradient>
<radialGradient id="potd" cx=".35" cy=".3" r=".8"><stop offset="0" stop-color="#4a2f18"/><stop offset=".6" stop-color="#1c120c"/><stop offset="1" stop-color="#070504"/></radialGradient>
<radialGradient id="globe" cx=".4" cy=".35" r=".75"><stop offset="0" stop-color="#b8742c"/><stop offset=".6" stop-color="#5a3214"/><stop offset="1" stop-color="#1c100a"/></radialGradient>
<radialGradient id="vg" cx=".5" cy=".5" r=".72"><stop offset=".55" stop-color="#000" stop-opacity="0"/><stop offset="1" stop-color="#000" stop-opacity=".85"/></radialGradient>
<filter id="soft" x="-60%" y="-60%" width="220%" height="220%"><feGaussianBlur stdDeviation="18"/></filter>
<filter id="b6" x="-20%" y="-20%" width="140%" height="140%"><feGaussianBlur stdDeviation="6"/></filter><filter id="b2"><feGaussianBlur stdDeviation="1.6"/></filter>
<filter id="glow" x="-80%" y="-80%" width="260%" height="260%"><feGaussianBlur stdDeviation="4" result="b"/><feMerge><feMergeNode in="b"/><feMergeNode in="SourceGraphic"/></feMerge></filter>
</defs>'''
p = [defs, f'<rect width="{W}" height="{H}" fill="url(#bg)"/>']
# far background: blurred glasshouse ribs, hanging lantern fruit, a seated giant
far = ''
for x in range(-100, 1800, 150): far += f'<path d="M{x},0 Q{x+90},260 {x+30},720" stroke="#1a110b" stroke-width="26" fill="none"/>'
far += '<path d="M1180,720 L1200,420 Q1320,250 1440,420 L1460,720Z" fill="#241710"/><circle cx="1320" cy="330" r="80" fill="#241710"/>'
for x, y, r in [(250, 150, 34), (470, 96, 22), (1090, 110, 40), (1480, 190, 26), (880, 60, 18)]:
    far += f'<path d="M{x},0 L{x},{y-r}" stroke="#1a110b" stroke-width="5"/><circle cx="{x}" cy="{y}" r="{r*2.2}" fill="#ffb060" opacity=".14"/><circle cx="{x}" cy="{y}" r="{r}" fill="#c9743a" opacity=".55"/>'
p.append(f'<g filter="url(#b6)">{far}</g>')
p.append(f'<path d="M700,0 L900,0 L1010,{GY} L590,{GY}Z" fill="url(#shaft)"/>')
# centre monument: a great striped tomato held by a curling vine
mon = (f'<path d="M800,{GY} C800,600 910,600 930,470 C960,300 800,330 790,200 C784,120 850,70 900,110 C930,136 912,176 880,168 C900,150 880,128 858,140 C826,160 836,220 880,270 C990,380 1010,560 900,{GY}Z" fill="#3a2212" {S()}/>'
       f'<path d="M812,{GY-10} C820,610 918,600 944,470 C968,330 830,330 806,210" stroke="#b8742c" stroke-width="5" fill="none" stroke-linecap="round" opacity=".8"/>'
       f'<circle cx="770" cy="420" r="108" fill="url(#globe)" {S()}/>')
for i in range(-3, 4): mon += f'<path d="M{770-math.sqrt(max(0,108**2-(i*28)**2)):.0f},{420+i*28} Q770,{420+i*28+10} {770+math.sqrt(max(0,108**2-(i*28)**2)):.0f},{420+i*28}" stroke="{K}" stroke-width="4" fill="none" opacity=".7"/>'
mon += f'<path d="M700,350 q30,-36 76,-34" stroke="#f2c078" stroke-width="7" fill="none" stroke-linecap="round" opacity=".7"/><path d="M748,312 l-30,-22 l34,4 l10,-30 l14,30 l34,-6 l-28,24Z" fill="#35502a" {S(6)}/>'
mon += f'<path d="M640,{GY} q10,-70 60,-70 h150 q50,0 60,70Z" fill="#2a180e" {S()}/><path d="M670,{GY-50} h190" stroke="#b8742c" stroke-width="4" opacity=".7"/>'
p.append(mon)
# left shrine and right tower, play-layer dark with amber trim
def trim(dp): return f'<path d="{dp}" stroke="#a8662c" stroke-width="5" fill="none" stroke-linecap="round" opacity=".85"/>'
left = (f'<path d="M-20,{GY} L-20,130 Q110,40 250,150 L262,{GY}Z" fill="#150d09" {S()}/><path d="M30,{GY} L30,470 Q110,360 196,470 L196,{GY}Z" fill="#2a160e" {S()}/>'
        + trim(f'M30,{GY} L30,470 Q110,360 196,470 L196,{GY}') + trim('M-10,150 Q110,66 244,160') + trim('M0,330 H250') + trim('M0,300 H250'))
for cx in (60, 150):
    left += f'<path d="M{cx},290 C{cx-50},230 {cx-50},170 {cx},120 C{cx+50},170 {cx+50},230 {cx},290Z" fill="#0d0806" {S(6)}/>' + trim(f'M{cx},270 C{cx-30},226 {cx-30},180 {cx},146') + trim(f'M{cx-14},236 q14,-16 28,0 M{cx-14},206 q14,-16 28,0')
left += f'<path d="M62,{GY-40} C40,{GY-110} 170,{GY-110} 160,{GY-40}Z" fill="#f3e3c4" {S()}/><circle cx="96" cy="{GY-74}" r="5" fill="{K}"/><circle cx="124" cy="{GY-74}" r="5" fill="{K}"/><path d="M40,{GY} C40,{GY-50} 180,{GY-50} 184,{GY}Z" fill="#5a2a3a" {S()}/>'
right = f'<path d="M1440,{GY} L1452,230 Q1540,120 1640,200 L1640,{GY}Z" fill="#150d09" {S()}/>' + trim(f'M1456,330 H1640') + trim('M1460,300 H1640') + trim(f'M1500,{GY} L1500,520 Q1570,440 1640,500') + trim('M1462,236 Q1540,140 1630,204')
p += [left, right]
# ground: a stone lip over heaps of jars
def heap(y0, y1, n, rmin, rmax, grad, blur=False):
    o = ''
    items = sorted([(R.uniform(-20, W + 20), R.uniform(y0, y1), R.uniform(rmin, rmax)) for _ in range(n)], key=lambda t: t[1])
    for x, y, r in items:
        o += f'<circle cx="{x:.0f}" cy="{y-r*1.02:.0f}" r="{r*.26:.0f}" fill="none" stroke="{K}" stroke-width="{r*.2:.0f}"/><circle cx="{x:.0f}" cy="{y:.0f}" r="{r:.0f}" fill="url(#{grad})" {S(5)}/><path d="M{x-r*.6:.0f},{y-r*.36:.0f} Q{x:.0f},{y-r*.56:.0f} {x+r*.6:.0f},{y-r*.36:.0f}" stroke="{K}" stroke-width="3" fill="none" opacity=".7"/>'
    return f'<g filter="url(#b2)">{o}</g>' if blur else o
p.append(f'<rect x="0" y="{GY}" width="{W}" height="{H-GY}" fill="#0d0806"/>')
p.append(heap(GY + 40, GY + 110, 70, 20, 34, 'pot'))
p.append(f'<path d="M-10,{GY-4} H{W+10} V{GY+22} H-10Z" fill="#2a180e" {S()}/><path d="M0,{GY+4} H{W}" stroke="#b8742c" stroke-width="4" opacity=".8"/>')
p.append(f'<rect x="0" y="{GY-120}" width="{W}" height="130" fill="#e8a858" opacity=".10" filter="url(#soft)"/>')

# ---- actors ----
sc = .6
# Louis charging in from the left
p.append(g(250, GY - 2, louis(), .42))
# Simon: lunge, crowbar swung through a tomato
simon = human((0, -132), 42, (70, 5, -55, -40), (96, 100, -70, -105), SIMON, (CROWBAR, 96), 'f', 4)
p.append(f'<path d="M400,{GY-230} C500,{GY-330} 720,{GY-330} 840,{GY-180}" stroke="#fff" stroke-width="22" fill="none" stroke-linecap="round" opacity=".28" filter="url(#b2)"/>')
p.append(f'<path d="M420,{GY-244} C530,{GY-346} 740,{GY-330} 850,{GY-172} C740,{GY-290} 560,{GY-300} 420,{GY-244}Z" fill="#fff" opacity=".92"/>')
p.append(g(880, GY - 150, burst(random.Random(4)), .7))
p.append(g(520, GY - 2, simon, sc))
# Charm: inverted mid-flip above a lunging beefsteak, sword arc trailing
p.append(f'<path d="M1010,300 A150,150 0 1 1 1262,404" stroke="#fff" stroke-width="30" fill="none" stroke-linecap="round" opacity=".22" filter="url(#b2)"/>')
p.append(f'<path d="M1004,292 A156,156 0 1 1 1270,410 A140,140 0 1 0 1004,292Z" fill="#fff" opacity=".9"/>')
charm = human((0, 0), 34, (105, -25, 62, -70), (60, 95, -150, -165), CHARM, (SWORD, 196), 'b', -10)
p.append(g(1150, 300, charm, sc, rot=205))
big = tomato(120, True)
p.append(g(1230, GY - 150, big, .78, rot=-16))
p.append(f'<path d="M1370,{GY-150} l60,-10 M1376,{GY-110} l70,6 M1366,{GY-70} l50,16" stroke="#fff6e0" stroke-width="5" stroke-linecap="round" opacity=".7"/>')
# a flyer diving from the upper left with a thorn spear
fly = (f'<path d="M-6,-10 L140,-190" stroke="{K}" stroke-width="16" stroke-linecap="round"/><path d="M-6,-10 L140,-190" stroke="#d9c08a" stroke-width="6" stroke-linecap="round"/><path d="M128,-206 L170,-226 L152,-184Z" fill="#f4fbff" {S(6)}/>'
       f'<path d="M-30,-40 C-110,-150 -200,-110 -230,-150 C-220,-70 -150,-20 -40,-10Z M40,-40 C90,-130 150,-110 190,-150 C190,-80 130,-30 50,-10Z" fill="#4c8a3c" {S()}/>' + tomato(56, False, legs=(40, 10)))
p.append(g(470, 250, fly, .62, rot=24, flip=True))
# foreground heap and atmosphere
p.append(heap(GY + 120, H + 30, 46, 34, 60, 'potd', True))
for i in range(90):
    x, y, r = R.uniform(0, W), R.uniform(0, H), R.uniform(1, 3.4)
    p.append(f'<circle cx="{x:.0f}" cy="{y:.0f}" r="{r:.1f}" fill="#ffe2b0" opacity="{R.uniform(.15,.7):.2f}" filter="url(#glow)"/>')
p.append(f'<rect width="{W}" height="{H}" fill="url(#vg)"/>')
# HUD
hud = f'<circle cx="92" cy="80" r="30" fill="#f3e3c4" {S(5)}/><circle cx="92" cy="80" r="12" fill="#d8392f" {S(4)}/><path d="M92,62 l-10,-8 l10,2 l4,-10 l4,10 l10,-2 l-8,8Z" fill="#4c8a3c" {S(3)}/>'
for i in range(6):
    fill = '#f3e3c4' if i < 4 else 'none'
    hud += f'<path transform="translate({150+i*40},80)" d="M0,-18 C14,-18 16,0 12,8 L0,20 L-12,8 C-16,0 -14,-18 0,-18Z" fill="{fill}" stroke="#f3e3c4" stroke-width="3"/>'
hud += f'<rect x="64" y="124" width="110" height="26" rx="13" fill="#f3e3c4" {S(5)}/><rect x="70" y="130" width="70" height="14" rx="7" fill="#fff"/>'
p.append(hud)
open('shot.svg', 'w').write(f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {W} {H}" width="{W}" height="{H}">{"".join(p)}</svg>')
