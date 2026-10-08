# Design pass 2: dark, sharp, dynamic. Writes sheet2.svg (characters) and frame2.svg (in-game target frame).
import math, random
INK = '#07070d'
def S(w=3.5): return f'stroke="{INK}" stroke-width="{w}" stroke-linejoin="round" stroke-linecap="round"'
def g(x, y, body, s=1.0, rot=0, flip=False):
    sx = -s if flip else s
    return f'<g transform="translate({x},{y}) rotate({rot}) scale({sx},{s})">{body}</g>'

DEFS = f'''<defs>
<filter id="glow" x="-80%" y="-80%" width="260%" height="260%"><feGaussianBlur stdDeviation="6" result="b"/><feMerge><feMergeNode in="b"/><feMergeNode in="b"/><feMergeNode in="SourceGraphic"/></feMerge></filter>
<filter id="soft" x="-50%" y="-50%" width="200%" height="200%"><feGaussianBlur stdDeviation="14"/></filter>
<filter id="blur4"><feGaussianBlur stdDeviation="4"/></filter>
<filter id="rim" x="-30%" y="-30%" width="160%" height="160%"><feDropShadow dx="0" dy="0" stdDeviation="5" flood-color="#9fd8ff" flood-opacity=".35"/></filter>
<pattern id="stripes" width="22" height="22" patternUnits="userSpaceOnUse" patternTransform="rotate(78)"><rect width="22" height="22" fill="#e9e4d6"/><rect width="11" height="22" fill="#18203c"/></pattern>
<pattern id="charm" width="20" height="20" patternUnits="userSpaceOnUse" patternTransform="rotate(20)"><rect width="20" height="20" fill="#a3201f"/><rect x="4" y="4" width="8" height="8" fill="none" stroke="#e8c9a0" stroke-width="2"/><circle cx="16" cy="16" r="1.8" fill="#e8c9a0"/></pattern>
<linearGradient id="blade" x1="0" x2="1"><stop offset="0" stop-color="#bfe9ff"/><stop offset="1" stop-color="#ffffff"/></linearGradient>
<linearGradient id="steel" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="#ff6a4a"/><stop offset="1" stop-color="#8c1d14"/></linearGradient>
<linearGradient id="coat" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#2c3760"/><stop offset="1" stop-color="#10142a"/></linearGradient>
<linearGradient id="fur" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#ecd0a0"/><stop offset="1" stop-color="#a97e4c"/></linearGradient>
<radialGradient id="t_red" cx=".35" cy=".3" r=".8"><stop offset="0" stop-color="#ff6b4a"/><stop offset=".55" stop-color="#c81f1a"/><stop offset="1" stop-color="#4a0a12"/></radialGradient>
<radialGradient id="t_dark" cx=".35" cy=".3" r=".8"><stop offset="0" stop-color="#b8402c"/><stop offset=".6" stop-color="#6e1712"/><stop offset="1" stop-color="#22060a"/></radialGradient>
<radialGradient id="t_orange" cx=".35" cy=".3" r=".8"><stop offset="0" stop-color="#ffc060"/><stop offset=".55" stop-color="#e86a12"/><stop offset="1" stop-color="#5a1a08"/></radialGradient>
<radialGradient id="t_green" cx=".35" cy=".3" r=".8"><stop offset="0" stop-color="#d6f27a"/><stop offset=".55" stop-color="#6fa82a"/><stop offset="1" stop-color="#18300e"/></radialGradient>
<linearGradient id="sky" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#070b18"/><stop offset=".55" stop-color="#12233a"/><stop offset="1" stop-color="#1c3a47"/></linearGradient>
<linearGradient id="rock" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#1d2a3c"/><stop offset="1" stop-color="#05070d"/></linearGradient>
</defs>'''

def eye(x, y, w=13, rot=0, col='#fff'):
    return f'<path transform="translate({x},{y}) rotate({rot})" d="M{-w},-1 L{w*.2},{-w*.45} L{w},1 L{w*.1},{w*.38}Z" fill="{col}" filter="url(#glow)"/>'

def hero():  # crowbar. origin at feet, facing right, mid-lunge
    skin = '#e9bd98'
    return f'''<g filter="url(#rim)">
    <path d="M6,-176 C-50,-205 -96,-158 -168,-190 L-150,-160 C-196,-158 -214,-132 -252,-142 L-222,-112 C-160,-96 -92,-128 -6,-146Z" fill="url(#stripes)" {S()}/>
    <path d="M-14,-84 L-74,-44 L-118,-6 L-126,-18 L-150,0 L-104,0 L-62,-30 L4,-66Z" fill="#0c0f1e" {S()}/>
    <path d="M-24,-158 L30,-162 L58,-104 L96,-62 L46,-76 L40,-36 L8,-74 L-34,-34 L-34,-92 L-70,-70 L-40,-118Z" fill="url(#coat)" {S()}/>
    <path d="M-24,-158 L30,-162 L40,-140 L-30,-134Z" fill="#3b4a7a" opacity=".6"/>
    <path d="M18,-82 L66,-52 L60,-8 L52,0 L96,0 L78,-12 L86,-58 L40,-92Z" fill="#0c0f1e" {S()}/>
    <path d="M-150,-292 q-34,-8 -30,-38 l12,2 q-2,18 22,24 L44,-150 L34,-136Z" fill="url(#steel)" {S()}/>
    <path d="M-140,-284 L30,-148" stroke="#ffd0b8" stroke-width="2" opacity=".7"/>
    <path d="M14,-150 L-22,-176 L-52,-214 L-38,-226 L-10,-192 L28,-166Z" fill="{skin}" {S()}/>
    <path d="M-8,-178 l-8,-12 M-2,-172 l-12,-16" stroke="#3a4a6a" stroke-width="2.5"/>
    <path d="M-62,-212 l14,-22 l22,14 l-14,22Z" fill="#141828" {S()}/>
    <ellipse cx="-2" cy="-204" rx="9" ry="13" fill="#c99a78" {S()}/><circle cx="-6" cy="-190" r="6" fill="none" stroke="#f4e3a8" stroke-width="3"/>
    <path d="M0,-200 C-8,-252 62,-262 72,-214 C76,-196 68,-182 56,-176 L44,-148 L24,-172 C8,-176 2,-186 0,-200Z" fill="{skin}" {S()}/>
    <path d="M20,-246 q22,-10 40,6" stroke="#fff" stroke-width="4" fill="none" opacity=".55" stroke-linecap="round"/>
    <path d="M32,-170 L58,-173 L47,-146Z" fill="#3a3640" {S(3)}/><path d="M43,-158 l4,10 l3,-10Z" fill="#b9b4bd"/>
    <path d="M24,-222 L66,-206" stroke="{INK}" stroke-width="7" stroke-linecap="round"/>
    {eye(50,-202,11,18)}
    <path d="M46,-184 l14,-1" stroke="{INK}" stroke-width="3" stroke-linecap="round"/></g>'''

def charm():  # sword. origin at feet, facing right, low thrust
    skin = '#9c5f38'
    return f'''<g filter="url(#rim)">
    <path d="M18,-196 C-30,-246 -104,-214 -176,-248 L-150,-212 C-206,-212 -236,-184 -276,-196 L-236,-160 C-282,-150 -296,-128 -320,-130 C-250,-104 -120,-128 -4,-158Z" fill="#15101a" {S()}/>
    <path d="M-60,-212 C-130,-200 -190,-176 -250,-178" stroke="#c9b083" stroke-width="7" fill="none" stroke-linecap="round"/>
    <path d="M-20,-86 L-92,-52 L-150,-34 L-176,-44 L-170,-18 L-140,-18 L-84,-34 L2,-66Z" fill="#0c0f1e" {S()}/>
    <path d="M-10,-154 L40,-156 L64,-112 L112,-92 L62,-84 L72,-40 L24,-72 L-8,-22 L-28,-78 L-96,-58 L-50,-112Z" fill="url(#charm)" {S()}/>
    <path d="M-28,-78 L-96,-58 L-50,-112Z M24,-72 L72,-40 L62,-84Z" fill="#000" opacity=".28"/>
    <path d="M22,-84 L74,-58 L76,-10 L66,0 L112,0 L94,-14 L98,-66 L46,-96Z" fill="#0c0f1e" {S()}/>
    <g filter="url(#glow)"><path d="M112,-142 L330,-176 L352,-174 L332,-160 L114,-124Z" fill="url(#blade)"/></g>
    <path d="M112,-142 L330,-176 L352,-174 L332,-160 L114,-124Z" fill="none" stroke="#5aa9d6" stroke-width="2"/>
    <path d="M120,-134 L334,-168" stroke="#5aa9d6" stroke-width="1.5"/>
    <path d="M104,-170 q24,34 10,72 q-22,-34 -10,-72Z" fill="#e8c05a" {S(3)}/><circle cx="110" cy="-134" r="6" fill="#d63a3a" {S(2)}/>
    <path d="M74,-140 l34,-4 l2,18 l-34,4Z" fill="#2a1a22" {S(3)}/><path d="M66,-136 l-12,2 l4,10 l10,-2Z" fill="#e8c05a" {S(3)}/>
    <path d="M30,-146 L70,-142 L96,-140 L96,-124 L66,-124 L26,-128Z" fill="{skin}" {S()}/>
    <path d="M8,-196 C2,-242 70,-248 76,-204 C78,-184 66,-164 46,-160 C26,-160 10,-174 8,-196Z" fill="{skin}" {S()}/>
    <path d="M4,-190 C-6,-256 84,-262 80,-200 C62,-226 34,-226 4,-190Z" fill="#15101a" {S()}/>
    <path d="M46,-244 q24,6 30,34" stroke="#c9b083" stroke-width="5" fill="none" stroke-linecap="round"/>
    <path d="M34,-206 L70,-194" stroke="{INK}" stroke-width="6" stroke-linecap="round"/>
    {eye(54,-190,11,16)}
    <path d="M52,-172 l12,0" stroke="{INK}" stroke-width="3" stroke-linecap="round"/></g>'''

def louis():  # full sprint, origin under the chest
    return f'''<g filter="url(#rim)">
    <path d="M-150,-96 C-200,-120 -250,-92 -290,-112" stroke="{INK}" stroke-width="12" fill="none" stroke-linecap="round"/><path d="M-150,-96 C-200,-120 -250,-92 -290,-112" stroke="#c99f66" stroke-width="5" fill="none" stroke-linecap="round"/>
    <path d="M-120,-84 L-190,-52 L-262,-58 L-286,-48 L-262,-40 L-186,-32 L-104,-58Z" fill="#a97e4c" {S()}/>
    <path d="M70,-70 L150,-34 L214,-44 L238,-34 L212,-26 L146,-14 L56,-50Z" fill="#a97e4c" {S()}/>
    <path d="M-160,-92 C-110,-130 -30,-118 40,-116 C84,-124 118,-100 124,-74 C96,-50 60,-56 26,-58 C-20,-36 -80,-46 -124,-60 C-150,-64 -164,-76 -160,-92Z" fill="url(#fur)" {S()}/>
    <path d="M-136,-80 L-206,-30 L-270,-22 L-292,-8 L-266,-6 L-196,-10 L-112,-56Z" fill="url(#fur)" {S()}/>
    <path d="M84,-66 L170,-56 L236,-76 L262,-70 L240,-58 L172,-34 L74,-46Z" fill="url(#fur)" {S()}/>
    <path d="M92,-110 L150,-150 L186,-152 L176,-122 L128,-74Z" fill="url(#fur)" {S()}/>
    <path d="M132,-140 L170,-126 L122,-96 C96,-110 80,-126 30,-150 C70,-140 100,-140 132,-140Z" fill="#b3261e" {S(3)}/>
    <path d="M150,-158 L204,-176 L286,-150 L294,-140 L282,-134 L220,-128 L172,-124Z" fill="url(#fur)" {S()}/>
    <path d="M176,-168 L112,-186 L160,-146Z" fill="#a97e4c" {S()}/>
    <path d="M286,-150 l10,6 l-10,8Z" fill="{INK}"/>
    <path d="M196,-164 L236,-156" stroke="{INK}" stroke-width="5" stroke-linecap="round"/>
    {eye(222,-150,9,14)}</g>'''

def blob(rx, ry, lobes=0, amp=0.0, flat=0.0, n=48):
    pts = []
    for i in range(n):
        a = 2 * math.pi * i / n
        r = 1 + amp * math.cos(lobes * a + 1.2)
        y = math.sin(a) * ry * r
        if y > 0: y *= (1 - flat)
        pts.append(f'{math.cos(a)*rx*r:.1f},{y:.1f}')
    return 'M' + ' L'.join(pts) + 'Z'

def calyx(r, n=7, seed=1):
    rnd = random.Random(seed); pts = []
    for i in range(n * 2):
        a = math.pi * 2 * i / (n * 2) - math.pi / 2
        rr = r * (1.0 + rnd.random() * .5) if i % 2 == 0 else r * .28
        pts.append(f'{math.cos(a)*rr:.1f},{math.sin(a)*rr*.55:.1f}')
    return 'M' + ' L'.join(pts) + 'Z'

def tomato(r, grad='t_red', sx=1.0, sy=1.0, lobes=0, amp=0.0, tilt=-12, legs='stride', eye_col='#ffe98a', leaf='#2f6b2a', extra='', back='', seed=1, mouth='snarl'):
    w, h = r * sx, r * sy
    lift = {'stride': h * .55, 'coil': h * .35, 'none': h * 1.3, 'stomp': h * .4}[legs]
    cy = -h - lift
    L = ''
    if legs == 'stride':
        L = f'<path d="M{-w*.3},{cy+h*.8} L{-w*.9},{-h*.3} L{-w*1.35},0 L{-w*.8},0 M{w*.3},{cy+h*.8} L{w*.75},{-h*.5} L{w*.6},0 L{w*1.2},0" fill="none" {S(6)}/>'
    elif legs == 'coil':
        L = f'<path d="M{-w*.4},{cy+h*.8} L{-w*1.1},{-h*.55} L{-w*.5},{-h*.2} L{-w*1.1},0 L{-w*.5},0 M{w*.4},{cy+h*.8} L{w*1.1},{-h*.55} L{w*.5},{-h*.2} L{w*1.1},0 L{w*1.6},0" fill="none" {S(6)}/>'
    elif legs == 'stomp':
        L = f'<path d="M{-w*.5},{cy+h*.8} L{-w*.75},{-h*.2} L{-w*.6},0 L{-w*1.05},0 M{w*.45},{cy+h*.8} L{w*.8},{-h*.25} L{w*.65},0 L{w*1.15},0" fill="none" {S(9)}/>'
    ex = w * .42
    face = eye(-ex * .55, 0, r * .3, 22, eye_col) + eye(ex * 1.25, 0, r * .3, -22 + 180, eye_col)
    brows = f'<path d="M{-ex*1.5},{-r*.34} L{-ex*.05},{-r*.08} M{ex*2.1},{-r*.34} L{ex*.7},{-r*.08}" stroke="{INK}" stroke-width="{max(4,r*.13)}" stroke-linecap="round"/>'
    m = ''
    if mouth == 'snarl':
        m = f'<path d="M{-w*.3},{h*.5} l{w*.14},{-h*.14} l{w*.14},{h*.12} l{w*.14},{-h*.12} l{w*.14},{h*.12} l{w*.14},{-h*.1}" fill="none" {S(max(3,r*.08))}/>'
    elif mouth == 'slit':
        m = f'<path d="M{-w*.15},{h*.52} l{w*.6},{-h*.08}" fill="none" {S(max(3,r*.08))}/>'
    body = f'''{back}<path d="{blob(w,h,lobes,amp,.12)}" fill="url(#{grad})" {S(4)}/>
    <path d="M{-w*.62},{-h*.5} q{w*.3},{-h*.42} {w*.7},{-h*.38}" stroke="#fff" stroke-width="{max(3,r*.09)}" fill="none" opacity=".45" stroke-linecap="round"/>
    <g transform="translate({w*.12},{-h*.2})">{brows}{face}</g>{m}
    <g transform="translate(0,{-h*.94}) rotate(8)"><path d="{calyx(r*.78,7,seed)}" fill="{leaf}" {S(3.5)}/><path d="M0,0 q4,-{r*.5} {r*.4},-{r*.62}" fill="none" stroke="{INK}" stroke-width="9" stroke-linecap="round"/><path d="M0,0 q4,-{r*.5} {r*.4},-{r*.62}" fill="none" stroke="{leaf}" stroke-width="4" stroke-linecap="round"/></g>{extra}'''
    return f'<g filter="url(#rim)">{L}<g transform="translate(0,{cy}) rotate({tilt})">{body}</g></g>'

def wing(sgn): return f'<path transform="scale({sgn},1)" d="M10,-4 C50,-80 110,-70 150,-96 L128,-52 L156,-40 L118,-24 L134,2 L92,-2 L60,22Z" fill="#2f6b2a" {S(3.5)}/><path transform="scale({sgn},1)" d="M14,-4 L128,-52 M40,-14 L118,-24" stroke="{INK}" stroke-width="2"/>'
TOMS = [
 ('CHERRY', 'swarm walker', lambda: tomato(26, seed=2)),
 ('BEEFSTEAK', 'heavy brute', lambda: tomato(62, 't_dark', 1.2, .92, 4, .07, -6, 'stomp', seed=3,
    extra=f'<path d="M-70,10 L-112,40 L-96,66 L-64,50Z M70,6 L118,26 L112,58 L78,50Z" fill="#6e1712" {S(4)}/>')),
 ('HOPPER', 'leaps at you', lambda: tomato(32, 't_orange', 1.0, .9, tilt=-20, legs='coil', seed=4, mouth='slit')),
 ('PLUM', 'charging lancer', lambda: tomato(30, sx=1.75, sy=.72, tilt=-8, seed=5, mouth='slit',
    extra=f'<path d="M50,-4 L128,-12 L52,12Z" fill="#e8e2d0" {S(3)}/>')),
 ('GREENIE', 'seed spitter', lambda: tomato(32, 't_green', leaf='#1c4a1a', eye_col='#fff', seed=6, tilt=-4,
    extra='<g filter="url(#glow)"><path d="M46,10 L96,2 L50,22Z M110,-2 l30,-6 l-26,14Z" fill="#d6f27a"/></g>')),
 ('VINE FLYER', 'dive bomber', lambda: tomato(28, legs='none', tilt=-24, seed=7, mouth='slit', back=wing(1) + wing(-1))),
 ('SUN-DRIED', 'armoured', lambda: tomato(36, 't_dark', 1.15, .78, 6, .1, -6, 'stomp', leaf='#5a5226', seed=8, mouth='slit',
    extra=f'<path d="M-36,-10 q16,10 4,26 M26,-22 q-12,14 2,28 M-16,22 q18,-10 36,2" stroke="#1a0406" stroke-width="3.5" fill="none"/>')),
 ('BIG TOM', 'boss', lambda: tomato(84, 't_red', 1.12, .95, 5, .05, -5, 'stomp', seed=9,
    extra=f'<path d="M-54,-96 L-44,-150 L-22,-112 L0,-164 L22,-112 L46,-150 L54,-96Z" fill="#e8c05a" {S(4)}/><path d="M-70,30 l18,-22 l10,26 l22,-30" stroke="#1a0406" stroke-width="3.5" fill="none"/>')),
]

def label(x, y, t, s, anchor='middle'):
    return (f'<text x="{x}" y="{y}" text-anchor="{anchor}" font-size="24" font-weight="800" letter-spacing="4" fill="#e9e4d6">{t}</text>'
            f'<text x="{x}" y="{y+24}" text-anchor="{anchor}" font-size="15" letter-spacing="1" fill="#7f95ad">{s}</text>')

def sheet():
    p = [DEFS, '<rect width="1800" height="1250" fill="url(#sky)"/>',
         '<ellipse cx="420" cy="420" rx="420" ry="260" fill="#3b6f8a" opacity=".18" filter="url(#soft)"/><ellipse cx="1280" cy="420" rx="460" ry="260" fill="#8a3b3b" opacity=".16" filter="url(#soft)"/>',
         '<text x="70" y="78" font-size="38" font-weight="800" letter-spacing="6" fill="#e9e4d6">CHARM ADVENTURE IN TOMATO LAND</text>',
         '<text x="70" y="108" font-size="16" letter-spacing="3" fill="#7f95ad">CHARACTER DESIGNS V2</text>',
         '<path d="M70,560 H1730" stroke="#2c4258" stroke-width="2"/>',
         g(380, 540, hero(), 1.25), label(330, 600, 'THE CROWBAR', 'player 1'),
         g(880, 540, charm(), 1.25), label(920, 600, 'CHARM', 'player 2, the sword'),
         g(1540, 548, louis(), .62), label(1540, 600, 'LOUIS', 'called with Y'),
         '<path d="M70,1140 H1730" stroke="#2c4258" stroke-width="2"/>']
    xs = [110, 300, 520, 720, 930, 1150, 1400, 1630]
    for (n, s, f), x in zip(TOMS, xs):
        p += [g(x, 1120, f(), 1.0, flip=True), label(x, 1180, n, s)]
    return f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1800 1250" width="1800" height="1250" font-family="DejaVu Sans, Arial, sans-serif">{"".join(p)}</svg>'

def frame():
    R = random.Random(7); W, H = 1600, 1000; p = [DEFS, f'<rect width="{W}" height="{H}" fill="url(#sky)"/>']
    # far: hanging lantern fruit and giant stalks
    for x, y, r in [(260, 250, 90), (700, 180, 60), (1180, 300, 120), (1460, 160, 50)]:
        p.append(f'<path d="M{x},0 Q{x+20},{y/2} {x},{y-r}" stroke="#0f1c2c" stroke-width="10" fill="none"/><circle cx="{x}" cy="{y}" r="{r*1.8}" fill="#ff5a3a" opacity=".10" filter="url(#soft)"/><circle cx="{x}" cy="{y}" r="{r}" fill="#5a1c22" opacity=".8" filter="url(#blur4)"/><circle cx="{x-r*.3}" cy="{y-r*.3}" r="{r*.35}" fill="#ff8a5a" opacity=".35" filter="url(#blur4)"/>')
    for x in [90, 480, 930, 1340]:
        p.append(f'<path d="M{x},{H} C{x+60},700 {x-80},380 {x+40},0 L{x+110},0 C{x+10},400 {x+150},700 {x+90},{H}Z" fill="#0f1c2c" opacity=".9"/>')
    for x in [300, 820, 1250]:
        p.append(f'<path d="M{x},0 L{x+160},0 L{x-140},{H} L{x-380},{H}Z" fill="#bfe9ff" opacity=".045"/>')
    # mid: broken arches
    p.append('<path d="M-20,1000 L-20,470 L60,440 L90,520 Q250,330 410,520 L440,430 L520,470 L520,1000Z M1080,1000 L1080,520 L1150,480 L1180,560 Q1330,400 1480,560 L1500,470 L1620,440 L1620,1000Z" fill="#0c1626"/>')
    p.append('<rect y="560" width="1600" height="260" fill="#4f8fa8" opacity=".12" filter="url(#soft)"/>')
    def slab(x, y, w, h, roots=True):
        top = [(x + i * w / 8, y + R.uniform(-7, 7)) for i in range(9)]
        bot = [(x + w - i * w / 6 + R.uniform(-12, 12), y + h * (.6 + .4 * math.sin(i * math.pi / 6)) + R.uniform(-6, 10)) for i in range(7)]
        d = 'M' + ' L'.join(f'{a:.0f},{b:.0f}' for a, b in top + bot) + 'Z'
        o = f'<path d="{d}" fill="url(#rock)" stroke="{INK}" stroke-width="4" stroke-linejoin="round"/>'
        o += '<path d="M' + ' L'.join(f'{a:.0f},{b:.0f}' for a, b in top) + '" stroke="#4fd0b0" stroke-width="6" fill="none" stroke-linecap="round" opacity=".85"/>'
        o += '<path d="M' + ' L'.join(f'{a:.0f},{b+7:.0f}' for a, b in top) + '" stroke="#8ff0d8" stroke-width="2" fill="none" opacity=".5"/>'
        for i in range(int(w / 16)):
            gx = x + R.uniform(4, w - 4); o += f'<path d="M{gx:.0f},{y+2:.0f} q{R.uniform(-8,8):.0f},-{R.uniform(10,26):.0f} {R.uniform(-12,12):.0f},-{R.uniform(18,38):.0f}" stroke="#2f9a84" stroke-width="2.5" fill="none" stroke-linecap="round"/>'
        if roots:
            for i in range(int(w / 70)):
                rx = x + R.uniform(20, w - 20); ln = R.uniform(40, 130)
                o += f'<path d="M{rx:.0f},{y+h*.7:.0f} q{R.uniform(-20,20):.0f},{ln/2:.0f} {R.uniform(-14,14):.0f},{ln:.0f}" stroke="#0a1220" stroke-width="4" fill="none" stroke-linecap="round"/>'
        return o
    p += [slab(-40, 800, 760, 260, False), slab(880, 840, 780, 220, False), slab(520, 560, 300, 60), slab(980, 420, 260, 56), slab(1330, 610, 320, 60), slab(120, 380, 230, 54)]
    p.append('<path d="M720,1000 L735,930 L760,1000 L790,900 L815,1000 L850,940 L880,1000Z" fill="#0a1220" stroke="#b3261e" stroke-width="2"/>')  # thorn pit
    # actors
    p += [g(1060, 846, tomato(62, 't_dark', 1.2, .92, 4, .07, -6, 'stomp', seed=3), .8, flip=True),
          g(1420, 612, tomato(26, seed=2), .9, flip=True), g(1530, 612, tomato(26, seed=12), .8, flip=True),
          g(1110, 300, tomato(28, legs='none', tilt=-24, seed=7, mouth='slit', back=wing(1) + wing(-1)), .7, flip=True),
          g(600, 566, tomato(32, 't_orange', 1.0, .9, tilt=-20, legs='coil', seed=4, mouth='slit'), .8, flip=True),
          g(250, 802, louis(), .42), g(520, 806, charm(), .62),
          g(800, 560, hero(), .6, rot=-14)]
    p.append('<path d="M705,300 Q930,330 960,560" stroke="#ff8a6a" stroke-width="10" fill="none" opacity=".5" filter="url(#glow)" stroke-linecap="round"/>')
    p.append('<path d="M560,716 L760,690" stroke="#bfe9ff" stroke-width="3" opacity=".6"/><path d="M430,740 L330,752 M440,700 L360,706" stroke="#bfe9ff" stroke-width="2" opacity=".35"/>')
    for i in range(70):
        x, y, r = R.uniform(0, W), R.uniform(0, H), R.uniform(1, 3.2)
        p.append(f'<circle cx="{x:.0f}" cy="{y:.0f}" r="{r:.1f}" fill="{R.choice(["#bfe9ff","#ffd9a0","#8ff0d8"])}" opacity="{R.uniform(.25,.8):.2f}" filter="url(#glow)"/>')
    # foreground silhouettes
    fg = 'M0,0 L1600,0 L1600,70 ' + ' '.join(f'L{1600-i*40},{R.uniform(20,60) if i%2 else R.uniform(70,150):.0f}' for i in range(1, 40)) + ' L0,90Z'
    p.append(f'<path d="{fg}" fill="#030409"/>')
    p.append('<path d="M0,1000 L0,700 C40,760 20,860 110,900 C60,930 120,960 200,1000Z M1600,1000 L1600,640 C1540,740 1580,860 1480,900 C1540,940 1470,970 1400,1000Z" fill="#030409"/>')
    for i in range(46):
        x = R.choice([R.uniform(0, 260), R.uniform(1360, 1600)]); hgt = R.uniform(60, 190)
        p.append(f'<path d="M{x:.0f},1000 q{R.uniform(-30,30):.0f},-{hgt*.6:.0f} {R.uniform(-50,50):.0f},-{hgt:.0f}" stroke="#030409" stroke-width="{R.uniform(4,9):.0f}" fill="none" stroke-linecap="round"/>')
    p.append('<rect width="1600" height="1000" fill="none" stroke="#000" stroke-width="240" opacity=".35" filter="url(#soft)"/>')
    # HUD
    for i in range(5):
        col = '#e9e4d6' if i < 4 else 'none'
        p.append(f'<path transform="translate({70+i*44},120)" d="M0,-18 C14,-18 16,0 12,10 L0,22 L-12,10 C-16,0 -14,-18 0,-18Z" fill="{col}" stroke="#e9e4d6" stroke-width="2.5"/>')
    return f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {W} {H}" width="{W}" height="{H}">{"".join(p)}</svg>'

open('sheet2.svg', 'w').write(sheet()); open('frame2.svg', 'w').write(frame())
