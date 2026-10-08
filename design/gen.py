# Generates the character design sheet (design/sheet.svg).
O = '#2b1b2e'  # outline
def g(x, y, body, s=1.0): return f'<g transform="translate({x},{y}) scale({s})">{body}</g>'
def label(x, y, title, sub): return (f'<text x="{x}" y="{y}" text-anchor="middle" font-size="22" font-weight="700" fill="{O}">{title}</text>'
                                     f'<text x="{x}" y="{y+22}" text-anchor="middle" font-size="14" fill="#5b4a5e">{sub}</text>')
ST = f'stroke="{O}" stroke-width="4" stroke-linejoin="round" stroke-linecap="round"'
def eye(x, y, r=7): return f'<ellipse cx="{x}" cy="{y}" rx="{r*0.75}" ry="{r}" fill="{O}"/><circle cx="{x+1.5}" cy="{y-2.5}" r="{r*0.3}" fill="#fff"/>'

def simon():
    skin = '#f2c9a6'
    return f'''
    <defs><clipPath id="tee"><path d="M-30,-98 Q-34,-60 -27,-44 L27,-44 Q34,-60 30,-98 Q0,-108 -30,-98Z"/></clipPath></defs>
    <ellipse cx="0" cy="2" rx="44" ry="7" fill="#0002"/>
    <rect x="-22" y="-26" width="16" height="22" rx="6" fill="{skin}" {ST}/><rect x="6" y="-26" width="16" height="22" rx="6" fill="{skin}" {ST}/>
    <path d="M-28,-6 h24 v6 h-26 q-3,-6 2,-6Z" fill="#23233a" {ST}/><path d="M4,-6 h24 q5,0 2,6 h-26Z" fill="#23233a" {ST}/>
    <path d="M-28,-50 h56 l2,28 h-26 l-4,-10 l-4,10 h-26Z" fill="#a9cbe6" {ST}/>
    <path d="M-30,-98 Q-34,-60 -27,-44 L27,-44 Q34,-60 30,-98 Q0,-108 -30,-98Z" fill="#fbf7ee"/>
    <g clip-path="url(#tee)" stroke="#1f2a4d" stroke-width="7"><path d="M-40,-92h80M-40,-78h80M-40,-64h80M-40,-50h80"/></g>
    <path d="M-30,-98 Q-34,-60 -27,-44 L27,-44 Q34,-60 30,-98 Q0,-108 -30,-98Z" fill="none" {ST}/>
    <path d="M-30,-94 q-16,10 -14,36" fill="none" stroke="{O}" stroke-width="15" stroke-linecap="round"/><path d="M-30,-94 q-16,10 -14,36" fill="none" stroke="{skin}" stroke-width="9" stroke-linecap="round"/>
    <path d="M-41,-84 q-4,6 -2,12" stroke="#55607a" stroke-width="3" fill="none"/>
    <path d="M30,-94 q18,8 22,30" fill="none" stroke="{O}" stroke-width="15" stroke-linecap="round"/><path d="M30,-94 q18,8 22,30" fill="none" stroke="{skin}" stroke-width="9" stroke-linecap="round"/>
    <g transform="translate(54,-62) rotate(35)"><rect x="-4" y="-6" width="8" height="34" rx="4" fill="#3a2b2b" {ST}/><ellipse cx="0" cy="-30" rx="24" ry="26" fill="#4a4a5a" {ST}/><ellipse cx="0" cy="-30" rx="16" ry="18" fill="#6c6c80"/></g>
    <path d="M-14,-100 q14,12 28,0" fill="none" stroke="#c9ccd6" stroke-width="3"/>
    <ellipse cx="-47" cy="-150" rx="8" ry="11" fill="{skin}" {ST}/><circle cx="-49" cy="-140" r="5" fill="none" stroke="#c9ccd6" stroke-width="3"/>
    <path d="M-46,-150 C-48,-212 48,-212 46,-150 C46,-112 20,-98 0,-98 C-20,-98 -46,-112 -46,-150Z" fill="{skin}" {ST}/>
    <path d="M-20,-190 q14,-12 34,-6" stroke="#fff" stroke-width="5" fill="none" opacity=".7" stroke-linecap="round"/>
    <path d="M-16,-128 q16,-10 34,0 q2,22 -17,28 q-19,-6 -17,-28Z" fill="#5a5560" {ST}/><path d="M-6,-108 q7,6 14,0 l-7,6Z" fill="#c9c6cc"/>
    <path d="M-6,-122 q8,6 18,0" stroke="#f6dcc6" stroke-width="4" fill="none" stroke-linecap="round"/>
    {eye(-10,-150)}{eye(22,-150)}
    <path d="M-20,-164 l14,3 M16,-161 l14,-3" stroke="{O}" stroke-width="4" stroke-linecap="round"/>'''

def wife():
    skin = '#a9693f'; hair = '#241a1c'
    return f'''
    <defs><pattern id="dress" width="18" height="18" patternUnits="userSpaceOnUse"><rect width="18" height="18" fill="#f6ead2"/><rect x="2" y="2" width="9" height="9" fill="none" stroke="#b2332b" stroke-width="2.5"/><circle cx="14" cy="14" r="2.2" fill="#b2332b"/></pattern></defs>
    <ellipse cx="0" cy="2" rx="46" ry="7" fill="#0002"/>
    <path d="M-20,-6 h16 v6 h-20Z M4,-6 h16 l4,6 h-20Z" fill="#d9c9a8" {ST}/>
    <path d="M-48,-152 C-58,-110 -50,-92 -40,-88 L40,-88 C50,-92 58,-110 48,-152Z" fill="{hair}" {ST}/>
    <path d="M-24,-98 Q-30,-60 -40,-8 L40,-8 Q30,-60 24,-98 Q0,-106 -24,-98Z" fill="url(#dress)" {ST}/>
    <path d="M-27,-66 h54" stroke="#b2332b" stroke-width="5"/>
    <path d="M-24,-94 q-18,12 -12,36" fill="none" stroke="{O}" stroke-width="14" stroke-linecap="round"/><path d="M-24,-94 q-18,12 -12,36" fill="none" stroke="{skin}" stroke-width="8" stroke-linecap="round"/>
    <path d="M24,-94 q20,6 30,22" fill="none" stroke="{O}" stroke-width="14" stroke-linecap="round"/><path d="M24,-94 q20,6 30,22" fill="none" stroke="{skin}" stroke-width="8" stroke-linecap="round"/>
    <path d="M-22,-98 L30,-52" stroke="{O}" stroke-width="8"/><path d="M-22,-98 L30,-52" stroke="#9a5632" stroke-width="4"/>
    <g transform="translate(58,-66) rotate(-20)"><path d="M0,0 q10,-22 20,-34" stroke="#9a5632" stroke-width="4" fill="none"/><rect x="2" y="-58" width="40" height="30" rx="6" fill="#9a5632" {ST}/><path d="M2,-48 h40" stroke="{O}" stroke-width="3"/></g>
    <path d="M-12,-100 q12,10 24,0" fill="none" stroke="#e2b54a" stroke-width="3"/>
    <path d="M-44,-150 C-46,-208 46,-208 44,-150 C44,-114 20,-100 0,-100 C-20,-100 -44,-114 -44,-150Z" fill="{skin}" {ST}/>
    <path d="M-46,-146 C-54,-222 54,-222 46,-146 C40,-176 20,-186 -4,-186 C-22,-184 -40,-172 -46,-146Z" fill="{hair}" {ST}/>
    <path d="M30,-196 q14,20 12,44" stroke="#bfa77a" stroke-width="6" fill="none" stroke-linecap="round"/>
    <path d="M-18,-128 q20,22 40,0Z" fill="#fff" {ST}/>
    <path d="M-16,-152 q6,-8 12,0 M16,-152 q6,-8 12,0" stroke="{O}" stroke-width="4.5" fill="none" stroke-linecap="round"/>
    <circle cx="-26" cy="-138" r="6" fill="#d9584a" opacity=".45"/><circle cx="34" cy="-138" r="6" fill="#d9584a" opacity=".45"/>'''

def louis():
    f = '#dcbd8c'; d = '#c29f6c'
    return f'''
    <ellipse cx="0" cy="2" rx="80" ry="7" fill="#0002"/>
    <path d="M-66,-74 q-34,6 -40,40" fill="none" stroke="{O}" stroke-width="10" stroke-linecap="round"/><path d="M-66,-74 q-34,6 -40,40" fill="none" stroke="{f}" stroke-width="4" stroke-linecap="round"/>
    <path d="M-52,-56 l-8,54 M-36,-52 l4,50 M34,-50 l-4,48 M50,-54 l6,52" stroke="{O}" stroke-width="13" stroke-linecap="round"/>
    <path d="M-52,-56 l-8,54 M34,-50 l-4,48" stroke="{d}" stroke-width="6" stroke-linecap="round"/><path d="M-36,-52 l4,50 M50,-54 l6,52" stroke="{f}" stroke-width="6" stroke-linecap="round"/>
    <path d="M-70,-78 Q-40,-96 10,-90 Q46,-96 62,-74 Q70,-40 40,-38 Q0,-44 -30,-60 Q-62,-52 -70,-78Z" fill="{f}" {ST}/>
    <path d="M40,-84 Q54,-120 66,-136 L92,-124 Q78,-96 62,-70Z" fill="{f}" {ST}/>
    <path d="M56,-110 l28,12" stroke="#d64a3a" stroke-width="8"/><circle cx="72" cy="-98" r="5" fill="#e2b54a" stroke="{O}" stroke-width="2"/>
    <path d="M60,-150 Q78,-170 100,-156 L146,-136 Q152,-126 142,-122 L96,-116 Q64,-118 60,-150Z" fill="{f}" {ST}/>
    <path d="M66,-158 q-16,8 -8,34 q14,-6 18,-28Z" fill="{d}" {ST}/>
    <ellipse cx="146" cy="-130" rx="7" ry="6" fill="{O}"/>
    {eye(104,-142,6)}
    <path d="M118,-120 q10,6 22,0" stroke="{O}" stroke-width="3" fill="none" stroke-linecap="round"/>'''

def tomato(r=34, col='#e23b2e', hi='#f4705c', sx=1.0, sy=1.0, brow='angry', legs='walk', extra='', leaf='#3f9b3a'):
    w, h = r*sx, r*sy
    cy = -h - 12
    b = f'<ellipse cx="0" cy="2" rx="{w*.8}" ry="5" fill="#0002"/>'
    if legs == 'walk':
        b += f'<path d="M{-w*.4},-14 v10 M{w*.4},-14 v10" stroke="{O}" stroke-width="6" stroke-linecap="round"/><path d="M{-w*.4-8},-2 h12 M{w*.4-4},-2 h12" stroke="{O}" stroke-width="7" stroke-linecap="round"/>'
    elif legs == 'spring':
        b += f'<path d="M{-w*.4},-16 l-7,5 l14,4 l-14,4 M{w*.4},-16 l7,5 l-14,4 l14,4" stroke="{O}" stroke-width="4" fill="none" stroke-linecap="round" stroke-linejoin="round"/>'
        cy -= 2
    elif legs == 'none':
        cy -= 26
    b += f'<ellipse cx="0" cy="{cy}" rx="{w}" ry="{h}" fill="{col}" {ST}/><ellipse cx="{-w*.4}" cy="{cy-h*.45}" rx="{w*.3}" ry="{h*.2}" fill="{hi}"/>'
    b += f'<path d="M0,{cy-h+2} l-16,-8 l12,10 l-14,6 l16,-2 l2,10 l4,-10 l16,2 l-14,-6 l12,-10Z" fill="{leaf}" {ST}/><path d="M0,{cy-h} v-10" stroke="{leaf}" stroke-width="5" stroke-linecap="round"/>'
    ex, ey = w*.38, cy - h*.05
    b += eye(-ex, ey, max(4, r*.17)) + eye(ex, ey, max(4, r*.17))
    if brow == 'angry': b += f'<path d="M{-ex-8},{ey-13} l14,6 M{ex+8},{ey-13} l-14,6" stroke="{O}" stroke-width="4" stroke-linecap="round"/>'
    if brow == 'sleepy': b += f'<path d="M{-ex-8},{ey-9} h16 M{ex-8},{ey-9} h16" stroke="{O}" stroke-width="4" stroke-linecap="round"/>'
    b += f'<path d="M{-w*.25},{cy+h*.45} q{w*.25},{-h*.25} {w*.5},0" stroke="{O}" stroke-width="4" fill="none" stroke-linecap="round"/>'
    return b + extra.replace('CY', str(cy))

wings = f'<path d="M-30,CY q-40,-40 -58,-6 q26,14 58,6Z M30,CY q40,-40 58,-6 q-26,14 -58,6Z" fill="#7fd06a" {ST}/>'
wrinkle = f'<g transform="translate(0,CY)"><path d="M-22,-14 q8,6 2,14 M18,-18 q-8,8 0,16 M-10,16 q10,-6 20,0" stroke="#5a1410" stroke-width="3" fill="none"/></g>'
crown = f'<g transform="translate(0,CY)"><path d="M-30,-66 l8,-26 l12,18 l10,-24 l10,24 l12,-18 l8,26Z" fill="#f2c23e" {ST}/></g>'
spit = f'<g transform="translate(0,CY)"><circle cx="50" cy="6" r="7" fill="#9bd24a" {ST}/><circle cx="68" cy="2" r="4" fill="#9bd24a" {ST}/></g>'

parts = ['<rect width="1600" height="900" fill="#fff6e3"/>',
 '<text x="60" y="64" font-size="34" font-weight="800" fill="#2b1b2e">Charm Adventure in Tomato Land: character designs v1</text>',
 '<text x="60" y="92" font-size="16" fill="#5b4a5e">For approval. Flat vector, thick outline, large heads. Each figure is built from separate parts for cutout animation.</text>',
 '<path d="M60,400 H1540" stroke="#e7d7b5" stroke-width="3"/>',
 g(260, 370, simon(), 1.25), label(260, 420, 'Player 1', 'striped tee, goatee, hoop earring, frying pan'),
 g(700, 370, wife(), 1.25), label(700, 420, 'Player 2', 'patterned dress, crossbody bag as weapon'),
 g(1200, 370, louis(), 1.25), label(1230, 420, 'Louis', 'fawn sighthound, red collar'),
 '<text x="60" y="500" font-size="24" font-weight="800" fill="#2b1b2e">Tomatoes</text>']
toms = [
 ('Cherry', 'small walker', tomato(24)),
 ('Beefsteak', 'big, slow, tough', tomato(46, '#c22a22', '#de5a4c', 1.15, 0.95)),
 ('Hopper', 'jumps at players', tomato(28, '#f28a1e', '#f9b25c', legs='spring', brow='none')),
 ('Plum', 'charges in a line', tomato(30, '#d8342c', '#ee6a58', 0.75, 1.25)),
 ('Greenie', 'unripe, spits seeds', tomato(28, '#8cc63f', '#b9e07a', brow='angry', extra=spit, leaf='#2f7d2c')),
 ('Vine Flyer', 'swoops from above', tomato(24, legs='none', extra=wings)),
 ('Sun-dried', 'armoured, slow', tomato(30, '#8e2a1e', '#a8463a', 1.1, 0.8, brow='sleepy', extra=wrinkle, leaf='#7a6a2c')),
 ('Big Tom', 'boss', tomato(58, '#d42c22', '#ee6654', 1.1, 1.0, extra=crown)),
]
xs = [130, 310, 500, 670, 850, 1040, 1230, 1440]
for (n, s, body), x in zip(toms, xs):
    parts += [g(x, 760, body, 1.25), label(x, 810, n, s)]
parts.append('<text x="60" y="870" font-size="15" fill="#5b4a5e">Open choices: weapons (frying pan, bag) are my proposal. Cherry, Beefsteak and Hopper exist in the build as boxes; the other five are proposals.</text>')
open('sheet.svg', 'w').write(f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1600 900" width="1600" height="900" font-family="DejaVu Sans, Arial, sans-serif">{"".join(parts)}</svg>')
