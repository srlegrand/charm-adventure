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
axes = {'simon': -113.9, 'charm': -65.6}
for name, parts in (('simon', simon), ('charm', charm)):
    ron.append(f'        "{name}": (\n            parts: {{')
    for role, (body, piv) in parts.items():
        render(part(body, W, H, OX, OY), f'{OUT}/{name}/{role}.png', W, H)
        ron.append(f'                "{role}": ({piv[0]*K+OX}, {piv[1]*K+OY}),')
    ron.append(f'            }},\n            weapon_axis: {axes[name]},\n        ),')
ron.append('    },\n    enemies: {')
# game-unit width of the hitbox the body should fill, per kind that exists in tuning.ron
for (n, _, body), (bodyw, hit) in zip(G.TOMS, [(48, 30), (145, 54), (60, 34), (45, 0), (60, 0), (52, 0), (75, 0), (176, 0)]):
    f = n.lower().replace(' ', '_').replace('-', '_')
    if f not in ('cherry', 'beefsteak', 'hopper'): render(part(body, 240, 240, 120, 228), f'{OUT}/tomatoes/{f}.png', 240, 240)
    upp = (hit * 1.25 / bodyw / K) if hit else 0.75 / K
    ron.append(f'        "{f}": (file: "sprites/tomatoes/{f}.png", size: (240, 240), origin: (120, 228), units_per_px: {upp:.4f}),')
ron.append('    },\n)')
open(f'{OUT}/rigs.ron', 'w').write('\n'.join(ron) + '\n')

# three-quarter tomatoes facing right, black outline finish (the game mirrors them for left)
import shot as T
def tq(name, sx, sy, body):
    render(part(f'<g transform="scale({-sx},{sy})">{body}</g>', 240, 240, 120, 228 - 42.5 * sy), f'{OUT}/tomatoes/{name}.png', 240, 240)
r = 60; Kc = T.K; St = T.S
mohawk = f'<path d="M{-.5*r},{-.9*r} L{-.74*r},{-1.5*r} L{-.3*r},{-1.12*r} L{-.2*r},{-1.74*r} L{.06*r},{-1.16*r} L{.34*r},{-1.6*r} L{.36*r},{-.95*r}Z" fill="#4c8a3c" {St()}/>'
fang = f'<path d="M{-.7*r},{.42*r} l{.07*r},{.26*r} l{.08*r},{-.24*r}Z M{-.1*r},{.3*r} l{.07*r},{.26*r} l{.08*r},{-.24*r}Z" fill="#fff6e0" {St(4)}/>'
tq('cherry', 1.0, 1.0, T.tomato(r, True, '#e8402f', '#a3202a', (28, -30)) + mohawk + fang)
arms = (f'<path d="M{.8*r},{.25*r} L{1.2*r},{.5*r}" stroke="{Kc}" stroke-width="26" stroke-linecap="round"/><circle cx="{1.24*r}" cy="{.54*r}" r="{.2*r}" fill="#8a1a22" {St()}/>')
arm_front = (f'<path d="M{-.78*r},{.34*r} L{-1.16*r},{.66*r}" stroke="{Kc}" stroke-width="26" stroke-linecap="round"/><circle cx="{-1.2*r}" cy="{.7*r}" r="{.22*r}" fill="#b8242a" {St()}/>')
grooves = f'<path d="M{.34*r},{-.94*r} Q{.6*r},0 {.34*r},{.94*r} M{-.34*r},{-.94*r} Q{-.5*r},{-.4*r} {-.46*r},{-.5*r}" stroke="{Kc}" stroke-width="5" fill="none" opacity=".55"/>'
tusk = f'<path d="M{-.68*r},{.66*r} L{-.62*r},{.2*r} L{-.5*r},{.62*r}Z M{-.04*r},{.56*r} L{.04*r},{.1*r} L{.14*r},{.5*r}Z" fill="#fff6e0" {St(5)}/>'
scar = f'<path d="M{.3*r},{-.7*r} L{.56*r},{-.3*r} M{.34*r},{-.44*r} l{.14*r},{-.1*r} M{.42*r},{-.58*r} l{.14*r},{-.1*r}" stroke="{Kc}" stroke-width="5" fill="none"/>'
tq('beefsteak', 1.2, .95, arms + T.tomato(r, True, '#b8242a', '#7a1424', (16, -20)) + grooves + tusk + scar + arm_front)
gog = ''
for gx, gy, gr in ((-.62 * r, -.24 * r, .27 * r), (.1 * r, -.2 * r, .31 * r)):
    gog += f'<circle cx="{gx}" cy="{gy}" r="{gr}" fill="#fff6e0" stroke="{Kc}" stroke-width="9"/><circle cx="{gx-gr*.3}" cy="{gy+gr*.1}" r="{gr*.36}" fill="{Kc}"/>'
gog += f'<path d="M{-.36*r},{-.26*r} L{-.2*r},{-.24*r} M{.42*r},{-.2*r} Q{.8*r},{-.3*r} {.98*r},{-.1*r}" stroke="{Kc}" stroke-width="9" fill="none"/>'
spring = f'<path d="M{-.4*r},{.8*r} l{-.3*r},{.12*r} l{.4*r},{.1*r} l{-.4*r},{.1*r} l{.3*r},{.1*r} M{.3*r},{.8*r} l{-.3*r},{.12*r} l{.4*r},{.1*r} l{-.4*r},{.1*r} l{.3*r},{.1*r}" stroke="{Kc}" stroke-width="9" fill="none" stroke-linecap="round" stroke-linejoin="round"/>'
tq('hopper', .95, 1.0, spring + T.tomato(r, False, '#f28a1e', '#b84a14', ()) + gog)

# backgrounds, 1600x1000, mirrored when tiled. Warm and dark, after the approved reference frame.
R = random.Random(11)
defs = T.defs if hasattr(T, 'defs') else ''
bgdefs = """<defs>
<radialGradient id="bg" cx=".5" cy=".42" r=".8"><stop offset="0" stop-color="#5a3e20"/><stop offset=".4" stop-color="#342313"/><stop offset=".85" stop-color="#150e0a"/><stop offset="1" stop-color="#0a0706"/></radialGradient>
<linearGradient id="shaft" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#ffdf9a" stop-opacity=".22"/><stop offset="1" stop-color="#ffdf9a" stop-opacity="0"/></linearGradient>
<filter id="soft" x="-60%" y="-60%" width="220%" height="220%"><feGaussianBlur stdDeviation="18"/></filter>
<filter id="b6" x="-20%" y="-20%" width="140%" height="140%"><feGaussianBlur stdDeviation="6"/></filter><filter id="b3" x="-20%" y="-20%" width="140%" height="140%"><feGaussianBlur stdDeviation="2.5"/></filter>
<filter id="glow" x="-80%" y="-80%" width="260%" height="260%"><feGaussianBlur stdDeviation="4" result="b"/><feMerge><feMergeNode in="b"/><feMergeNode in="SourceGraphic"/></feMerge></filter>
<radialGradient id="vg" cx=".5" cy=".5" r=".75"><stop offset=".55" stop-color="#000" stop-opacity="0"/><stop offset="1" stop-color="#000" stop-opacity=".8"/></radialGradient>
<radialGradient id="pot" cx=".35" cy=".3" r=".8"><stop offset="0" stop-color="#8a5a2c"/><stop offset=".5" stop-color="#3a2414"/><stop offset="1" stop-color="#120b08"/></radialGradient>
<radialGradient id="fruit" cx=".4" cy=".35" r=".75"><stop offset="0" stop-color="#ffd08a"/><stop offset=".5" stop-color="#e8823a"/><stop offset="1" stop-color="#8a3414"/></radialGradient>
</defs>"""
far = [bgdefs, '<rect width="1600" height="1000" fill="url(#bg)"/>']; blur = ''
for x in range(-100, 1800, 160): blur += f'<path d="M{x},0 Q{x+90},300 {x+30},1000" stroke="#1a110b" stroke-width="28" fill="none"/>'
blur += '<path d="M1080,1000 L1100,560 Q1220,380 1340,560 L1360,1000Z" fill="#241710"/><circle cx="1220" cy="470" r="84" fill="#241710"/>'
blur += '<path d="M260,1000 C260,760 420,700 430,560 C440,430 330,420 330,330 C330,270 390,250 420,290 C400,270 370,290 384,330 C410,400 520,440 500,600 C480,760 380,820 380,1000Z" fill="#20140d"/><circle cx="330" cy="560" r="70" fill="#20140d"/>'
for x, y, r in [(250, 190, 34), (520, 110, 22), (900, 150, 40), (1440, 230, 26), (760, 70, 16)]:
    blur += f'<path d="M{x},0 L{x},{y-r}" stroke="#1a110b" stroke-width="5"/><circle cx="{x}" cy="{y}" r="{r*2.2}" fill="#ffb060" opacity=".13"/><circle cx="{x}" cy="{y}" r="{r}" fill="#c9743a" opacity=".5"/>'
far.append(f'<g filter="url(#b6)">{blur}</g>')
for x in [560, 1180]: far.append(f'<path d="M{x},0 L{x+170},0 L{x+260},1000 L{x-110},1000Z" fill="url(#shaft)"/>')
for i in range(60):
    far.append(f'<circle cx="{R.uniform(0,1600):.0f}" cy="{R.uniform(0,1000):.0f}" r="{R.uniform(1,3):.1f}" fill="#ffe2b0" opacity="{R.uniform(.15,.6):.2f}" filter="url(#glow)"/>')
render(f'<svg xmlns="http://www.w3.org/2000/svg" width="1600" height="1000">{"".join(far)}</svg>', f'{OUT}/bg/far.png', 1600, 1000)
mid = [bgdefs, '<g filter="url(#b3)"><path d="M0,1000 L0,430 Q130,300 270,440 L282,1000Z M1300,1000 L1312,500 Q1410,390 1520,470 L1600,520 L1600,1000Z" fill="#120b08"/>'
       '<path d="M40,1000 L40,640 Q130,540 220,640 L220,1000Z" fill="#1c110b"/><path d="M0,560 H280 M0,600 H280 M1310,560 H1600" stroke="#6a4020" stroke-width="5" opacity=".7"/>'
       '<path d="M640,1000 L690,700 L760,790 L800,640 L880,1000Z M930,1000 L960,820 L1010,1000Z" fill="#120b08"/></g>',
       '<rect x="-100" y="640" width="1800" height="260" fill="#e8a858" opacity=".09" filter="url(#soft)"/>']
render(f'<svg xmlns="http://www.w3.org/2000/svg" width="1600" height="1000">{"".join(mid)}</svg>', f'{OUT}/bg/mid.png', 1600, 1000)
render(f'<svg xmlns="http://www.w3.org/2000/svg" width="640" height="400">{bgdefs}<rect width="640" height="400" fill="url(#vg)"/></svg>', f'{OUT}/bg/vignette.png', 640, 400)

# props, effects and interface pieces
Kc = T.K; St = T.S
def prop(name, w, h, body): render(f'<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}">{bgdefs}{body}</svg>', f'{OUT}/{name}.png', w, h)
prop('props/pot', 96, 110, f'<circle cx="48" cy="22" r="10" fill="none" stroke="{Kc}" stroke-width="8"/><circle cx="48" cy="66" r="38" fill="url(#pot)" {St(6)}/><path d="M24,50 Q48,38 72,50" stroke="{Kc}" stroke-width="4" fill="none" opacity=".7"/><path d="M26,44 q12,-14 30,-14" stroke="#e8b070" stroke-width="5" fill="none" stroke-linecap="round" opacity=".7"/>')
prop('props/lantern', 96, 200, f'<path d="M48,0 Q56,60 48,112" stroke="{Kc}" stroke-width="9" fill="none"/><path d="M48,0 Q56,60 48,112" stroke="#35502a" stroke-width="4" fill="none"/><circle cx="48" cy="150" r="36" fill="url(#fruit)" {St(6)}/><path d="M48,116 l-22,-6 l14,12 l-10,14 l18,-10 l18,10 l-10,-14 l14,-12Z" fill="#4c8a3c" {St(5)}/><path d="M30,138 q8,-14 24,-14" stroke="#fff3d0" stroke-width="6" fill="none" stroke-linecap="round" opacity=".8"/>')
vine = f'<path d="M40,0 C60,60 20,110 44,170 C58,210 34,240 40,270" stroke="{Kc}" stroke-width="10" fill="none" stroke-linecap="round"/><path d="M40,0 C60,60 20,110 44,170 C58,210 34,240 40,270" stroke="#35502a" stroke-width="4" fill="none" stroke-linecap="round"/>'
for (x, y, s, a) in [(48, 50, 1, -30), (30, 108, -1, 20), (46, 170, 1, -10), (38, 232, -1, 30)]:
    vine += f'<path transform="translate({x},{y}) rotate({a}) scale({s},1)" d="M0,0 C14,-16 34,-12 40,4 C26,14 10,12 0,0Z" fill="#4c8a3c" {St(5)}/>'
prop('props/vine', 96, 280, vine)
prop('props/stalactite', 80, 150, f'<path d="M4,0 L76,0 L62,40 L52,34 L40,146 L30,50 L18,58Z" fill="#1c110b" {St(6)}/><path d="M16,6 L28,46 M56,6 L50,30" stroke="#6a4020" stroke-width="4" fill="none" opacity=".8"/>')
prop('props/sprout', 80, 100, f'<path d="M40,100 C40,70 36,50 40,34" stroke="{Kc}" stroke-width="10" fill="none"/><path d="M40,100 C40,70 36,50 40,34" stroke="#4c8a3c" stroke-width="4" fill="none"/><path d="M40,40 C20,36 8,20 10,6 C28,8 40,22 40,40Z M40,48 C56,40 72,26 70,10 C54,14 42,30 40,48Z" fill="#4c8a3c" {St(5)}/><circle cx="52" cy="66" r="10" fill="#d8392f" {St(5)}/>')
prop('props/arch', 400, 520, f'<g opacity=".9"><path d="M10,520 L10,220 Q200,-40 390,220 L390,520 L320,520 L320,250 Q200,90 80,250 L80,520Z" fill="#170e09"/><path d="M26,520 L26,226 Q200,-10 374,226 L374,520" stroke="#5a3418" stroke-width="6" fill="none"/><path d="M10,300 H80 M320,300 H390 M10,330 H80 M320,330 H390" stroke="#5a3418" stroke-width="5"/></g>')
prop('props/curl', 300, 600, f'<g opacity=".9"><path d="M110,600 C110,440 230,420 240,300 C250,170 130,180 130,90 C130,30 190,10 220,50 C200,30 170,50 184,90 C210,160 330,200 310,340 C290,470 200,480 200,600Z" fill="#170e09"/><path d="M124,596 C130,450 240,430 254,300 C262,200 150,190 142,100" stroke="#5a3418" stroke-width="5" fill="none"/><circle cx="120" cy="330" r="74" fill="#1c110b"/><path d="M60,300 Q120,316 180,300 M52,340 Q120,356 188,340 M66,376 Q120,390 174,376" stroke="#5a3418" stroke-width="4" fill="none"/></g>')
prop('fx/heart', 96, 96, f'<path d="M48,84 C10,56 6,30 22,18 C36,8 46,18 48,28 C50,18 60,8 74,18 C90,30 86,56 48,84Z" fill="#ff5a7a" {St(7)}/><path d="M24,30 q6,-10 16,-6" stroke="#ffd0dc" stroke-width="6" fill="none" stroke-linecap="round"/>')
prop('fx/ring', 256, 256, '<circle cx="128" cy="128" r="112" fill="none" stroke="#fff" stroke-width="18"/><circle cx="128" cy="128" r="96" fill="none" stroke="#fff" stroke-width="4" opacity=".5"/>')
prop('fx/wedge', 96, 72, f'<path d="M8,50 Q48,-30 88,50 Q48,70 8,50Z" fill="#d8392f" {St(6)}/><path d="M24,48 Q48,4 72,48 Q48,58 24,48Z" fill="#f58a6a"/><ellipse cx="40" cy="42" rx="4" ry="7" fill="#fff6e0"/><ellipse cx="58" cy="42" rx="4" ry="7" fill="#fff6e0"/>')
prop('fx/leafcap', 96, 64, f'<path d="M48,30 l-40,8 l26,10 l-16,14 l32,-12 l12,16 l12,-18 l32,8 l-22,-16 l22,-14 l-34,6 l-8,-26 l-10,26Z" fill="#4c8a3c" {St(6)}/>')
prop('ui/bubble', 96, 96, f'<rect x="6" y="6" width="84" height="84" rx="26" fill="#fff8ea" stroke="{Kc}" stroke-width="7"/>')
prop('ui/tail', 48, 48, f'<path d="M6,2 L42,2 L20,44Z" fill="#fff8ea" stroke="{Kc}" stroke-width="7" stroke-linejoin="round"/><rect x="8" y="0" width="32" height="7" fill="#fff8ea"/>')
prop('ui/wrap', 64, 80, f'<path d="M14,6 L50,6 L40,76 L24,76Z" fill="#e9d2a0" {St(6)}/><path d="M16,30 Q32,40 48,30" stroke="#b3262a" stroke-width="7" fill="none"/>')
render(f'<svg xmlns="http://www.w3.org/2000/svg" width="128" height="128"><defs><radialGradient id="l"><stop offset="0" stop-color="#fff" stop-opacity=".9"/><stop offset=".4" stop-color="#fff" stop-opacity=".3"/><stop offset="1" stop-color="#fff" stop-opacity="0"/></radialGradient></defs><rect width="128" height="128" fill="url(#l)"/></svg>', f'{OUT}/fx/light.png', 128, 128)
print('ok')
