# Exports game sprites from the approved designs (gen3.py): rig parts for Simon and Charm,
# whole tomatoes, background layers, and assets/sprites/rigs.ron with pivots.
import os, subprocess, glob, random, math
import gen3 as G
from gen3 import S, P, DK, arm, almond
OUT = '../assets/sprites'; K = 0.5
CH = glob.glob('/opt/pw-browsers/chromium*/chrome-linux*/chrome')[0]
def render(svg, path, w, h):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    open('/tmp/_p.svg', 'w').write(svg)
    subprocess.run([CH, '--headless', '--no-sandbox', '--disable-gpu', '--hide-scrollbars', '--default-background-color=00000000',
                    f'--window-size={w},{h+100}', f'--screenshot=/tmp/_p.png', 'file:///tmp/_p.svg'], capture_output=True)
    subprocess.run(['convert', '/tmp/_p.png', '-crop', f'{w}x{h}+0+0', '+repage', path], check=True)
def part(body, w, h, ox, oy):
    return f'<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}">{G.DEFS}<filter id="rim" x="-40%" y="-40%" width="180%" height="180%"><feDropShadow dx="0" dy="0" stdDeviation="2.2" flood-color="#bfe9ff" flood-opacity=".7"/></filter><g filter="url(#rim)"><g transform="translate({ox},{oy}) scale({K})">{body}</g></g></svg>'

sk = '#f3d2b3'
simon = {
 'leg_b': (f'<path d="M-20,-200 L0,-200 L-30,0Z" fill="url(#leg)" {S(4)}/>', (-10, -200)),
 'leg_f': (f'<path d="M6,-200 L26,-200 L40,0Z" fill="url(#leg)" {S(4)}/>', (16, -200)),
 'body': (f'''<path d="M-10,-360 L10,-360 L8,-330 L-8,-330Z" fill="{DK}" {S(4)}/>
    <path d="M-18,-344 L-66,-372 L-52,-322 C-72,-262 -88,-214 -104,-166 L-58,-190 L-34,-150 L-4,-188 L26,-152 L52,-190 L96,-164 C82,-220 64,-272 52,-322 L66,-372 L18,-344Z" fill="url(#br)" {S()}/>
    <path d="M52,-322 C64,-272 82,-220 96,-164 L52,-190 L26,-152 L18,-200 C30,-250 40,-290 52,-322Z" fill="{P}" opacity=".28"/>
    <path d="M-18,-344 L-66,-372 L-52,-322 L-20,-318Z M18,-344 L66,-372 L52,-322 L20,-318Z" fill="{DK}" {S(4)}/>
    {arm("M50,-318 L96,-268 L46,-232")}<circle cx="46" cy="-232" r="11" fill="{sk}" {S(4)}/>''', (0, -200)),
 'weapon': (f'''<path d="M-66,-262 L-176,-34 q-10,24 12,32 l8,-12 q-10,-6 -4,-16 L-52,-256Z" fill="#e2452e" {S(4)}/>
    <path d="M-62,-250 L-166,-36" stroke="#ff9a7a" stroke-width="3" stroke-linecap="round"/>
    {arm("M-50,-318 L-92,-262 L-78,-222")}<circle cx="-76" cy="-224" r="11" fill="{sk}" {S(4)}/>''', (-50, -318)),
 'head': (f'''<ellipse cx="-50" cy="-424" rx="9" ry="13" fill="#e2b894" {S(4)}/><circle cx="-54" cy="-408" r="7" fill="none" stroke="#f2b24a" stroke-width="4"/>
    <path d="M-46,-428 C-52,-506 52,-506 46,-428 C44,-398 22,-380 0,-352 C-22,-380 -44,-398 -46,-428Z" fill="{sk}" {S()}/>
    <path d="M-27,-388 C-10,-396 10,-396 27,-388 C16,-374 8,-364 0,-352 C-8,-364 -16,-374 -27,-388Z" fill="#4a3a52"/><path d="M-5,-368 L5,-368 L0,-354Z" fill="#cfc6d2"/>
    <path d="M-46,-428 C-52,-506 52,-506 46,-428 C44,-398 22,-380 0,-352 C-22,-380 -44,-398 -46,-428Z" fill="none" {S()}/>
    <path d="M22,-470 C34,-462 40,-446 40,-430 C28,-440 20,-454 22,-470Z" fill="#fff" opacity=".75"/>
    {almond(-20,-424,15,10,24)}{almond(20,-424,15,10,156)}''', (0, -356)),
}
sk2 = '#a9683f'; hair = '#2a1024'
charm = {
 'leg_b': simon['leg_b'], 'leg_f': simon['leg_f'],
 'back': (f'<path d="M-30,-486 C-64,-470 -66,-410 -56,-372 C-62,-330 -58,-290 -78,-262 C-50,-268 -34,-300 -34,-340 L34,-340 C34,-300 44,-262 70,-240 C76,-290 60,-340 56,-372 C66,-410 64,-470 30,-486Z" fill="{hair}" {S()}/>', (0, -356)),
 'body': (f'''<path d="M-10,-360 L10,-360 L8,-330 L-8,-330Z" fill="{DK}" {S(4)}/>
    <path d="M-14,-346 C-34,-300 -70,-226 -92,-176 C-70,-160 -48,-176 -30,-158 C-10,-176 12,-176 30,-158 C48,-176 70,-160 92,-176 C70,-226 34,-300 14,-346Z" fill="url(#ch)" {S()}/>
    <path d="M14,-346 C34,-300 70,-226 92,-176 C70,-160 48,-176 30,-158 C30,-220 26,-290 14,-346Z" fill="{P}" opacity=".3"/>
    <path d="M-30,-158 C-28,-220 -18,-290 -6,-342" stroke="{P}" stroke-width="3" fill="none" opacity=".6"/>
    {arm("M-22,-322 L-56,-280 L-40,-246", 8)}<circle cx="-40" cy="-246" r="10" fill="{sk2}" {S(4)}/>''', (0, -200)),
 'weapon': (f'''<path d="M62,-262 L70,-274 L170,4Z" fill="#eaf6ff" {S(4)}/><path d="M70,-262 L164,-4" stroke="#9fd0ea" stroke-width="2.5"/>
    {arm("M22,-322 L60,-296 L62,-272", 8)}
    <path d="M44,-258 q22,-30 42,-8" fill="none" stroke="{P}" stroke-width="12" stroke-linecap="round"/><path d="M44,-258 q22,-30 42,-8" fill="none" stroke="#f2b24a" stroke-width="6" stroke-linecap="round"/>
    <path d="M60,-278 L48,-312" stroke="{P}" stroke-width="10" stroke-linecap="round"/><circle cx="46" cy="-320" r="9" fill="none" stroke="{P}" stroke-width="5"/><circle cx="46" cy="-320" r="9" fill="none" stroke="#f2b24a" stroke-width="2"/>''', (22, -322)),
 'head': (f'''<path d="M-46,-428 C-52,-506 52,-506 46,-428 C44,-398 22,-380 0,-352 C-22,-380 -44,-398 -46,-428Z" fill="{sk2}" {S()}/>
    <path d="M-50,-420 C-58,-514 58,-514 50,-420 C34,-462 -4,-472 -50,-420Z" fill="{hair}" {S(4)}/>
    <path d="M22,-494 C46,-476 50,-440 58,-390" stroke="#d9c08a" stroke-width="7" fill="none" stroke-linecap="round"/>
    {almond(-20,-420,15,10,24,'#fff',False)}{almond(20,-420,15,10,156,'#fff',False)}''', (0, -356)),
}
W, H, OX, OY = 240, 280, 120, 265
UPP = (84 / 506) / K   # game units per PNG pixel: figures stand 84 units tall
ron = [f'// Written by design/export.py. Pivots are PNG pixels. Angles are degrees, for a figure facing right.\n(\n    canvas: ({W}, {H}),\n    origin: ({OX}, {OY}),\n    units_per_px: {UPP:.4f},\n    characters: {{']
swings = {'simon': ((-150, -250), (-120, -210), (60, -20)), 'charm': ((120, 40), (180, 110), (-50, 20))}
for name, parts in (('simon', simon), ('charm', charm)):
    ron.append(f'        "{name}": (\n            parts: {{')
    for role, (body, piv) in parts.items():
        render(part(body, W, H, OX, OY), f'{OUT}/{name}/{role}.png', W, H)
        ron.append(f'                "{role}": ({piv[0]*K+OX}, {piv[1]*K+OY}),')
    a, b, c = swings[name]
    ron.append(f'            }},\n            swing_side: {a},\n            swing_up: {b},\n            swing_down: {c},\n        ),')
ron.append('    },\n    enemies: {')
# game-unit width of the hitbox the body should fill, per kind that exists in tuning.ron
for (n, _, body), (bodyw, hit) in zip(G.TOMS, [(48, 30), (145, 54), (60, 34), (45, 0), (60, 0), (52, 0), (75, 0), (176, 0)]):
    f = n.lower().replace(' ', '_').replace('-', '_')
    render(part(body, 240, 240, 120, 228), f'{OUT}/tomatoes/{f}.png', 240, 240)
    upp = (hit * 1.25 / bodyw / K) if hit else 0.75 / K
    ron.append(f'        "{f}": (file: "sprites/tomatoes/{f}.png", size: (240, 240), origin: (120, 228), units_per_px: {upp:.4f}),')
ron.append('    },\n)')
open(f'{OUT}/rigs.ron', 'w').write('\n'.join(ron) + '\n')

# backgrounds, 1600x1000, mirrored when tiled
R = random.Random(11)
defs = '''<defs><linearGradient id="sky" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#070b18"/><stop offset=".55" stop-color="#12233a"/><stop offset="1" stop-color="#1c3a47"/></linearGradient>
<filter id="soft" x="-50%" y="-50%" width="200%" height="200%"><feGaussianBlur stdDeviation="16"/></filter><filter id="b4"><feGaussianBlur stdDeviation="4"/></filter>
<filter id="glow" x="-80%" y="-80%" width="260%" height="260%"><feGaussianBlur stdDeviation="5" result="b"/><feMerge><feMergeNode in="b"/><feMergeNode in="SourceGraphic"/></feMerge></filter>
<radialGradient id="vg" cx=".5" cy=".5" r=".75"><stop offset=".55" stop-color="#000" stop-opacity="0"/><stop offset="1" stop-color="#000" stop-opacity=".8"/></radialGradient></defs>'''
far = [defs, '<rect width="1600" height="1000" fill="url(#sky)"/>']
for x, y, r in [(300, 260, 90), (760, 170, 55), (1180, 330, 120)]:
    far.append(f'<path d="M{x},0 Q{x+20},{y/2} {x},{y-r}" stroke="#0f1c2c" stroke-width="10" fill="none"/><circle cx="{x}" cy="{y}" r="{r*1.8}" fill="#ff5a3a" opacity=".10" filter="url(#soft)"/><circle cx="{x}" cy="{y}" r="{r}" fill="#5a1c22" opacity=".8" filter="url(#b4)"/><circle cx="{x-r*.3}" cy="{y-r*.3}" r="{r*.35}" fill="#ff8a5a" opacity=".35" filter="url(#b4)"/>')
for x in [520, 960, 1400]:
    far.append(f'<path d="M{x},1000 C{x+60},700 {x-80},380 {x+40},0 L{x+110},0 C{x+10},400 {x+150},700 {x+90},1000Z" fill="#0f1c2c" opacity=".9"/>')
for x in [420, 900, 1300]:
    far.append(f'<path d="M{x},0 L{x+160},0 L{x-140},1000 L{x-380},1000Z" fill="#bfe9ff" opacity=".045"/>')
for i in range(50):
    far.append(f'<circle cx="{R.uniform(0,1600):.0f}" cy="{R.uniform(0,1000):.0f}" r="{R.uniform(1,3):.1f}" fill="{R.choice(["#bfe9ff","#ffd9a0","#8ff0d8"])}" opacity="{R.uniform(.2,.7):.2f}" filter="url(#glow)"/>')
render(f'<svg xmlns="http://www.w3.org/2000/svg" width="1600" height="1000">{"".join(far)}</svg>', f'{OUT}/bg/far.png', 1600, 1000)
mid = [defs, '<path d="M0,1000 L0,520 L70,470 L110,560 Q270,360 430,560 L460,460 L560,500 L600,1000Z M1000,1000 L1040,540 L1150,490 L1180,580 Q1330,400 1480,580 L1510,480 L1600,520 L1600,1000Z" fill="#0b1524"/>',
       '<path d="M640,1000 L700,760 L760,820 L800,700 L860,1000Z" fill="#0b1524"/>',
       '<rect x="-100" y="600" width="1800" height="260" fill="#4f8fa8" opacity=".14" filter="url(#soft)"/>']
render(f'<svg xmlns="http://www.w3.org/2000/svg" width="1600" height="1000">{"".join(mid)}</svg>', f'{OUT}/bg/mid.png', 1600, 1000)
render(f'<svg xmlns="http://www.w3.org/2000/svg" width="640" height="400">{defs}<rect width="640" height="400" fill="url(#vg)"/></svg>', f'{OUT}/bg/vignette.png', 640, 400)
print('ok')
