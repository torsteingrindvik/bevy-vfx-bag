/*
From: https://www.shadertoy.com/view/Xs3fR4

// variant of Vorocracks: https://shadertoy.com/view/lsVyRy
// integrated with cracks here: https://www.shadertoy.com/view/Xd3fRN


float ofs = 0.;


float RATIO = 1.,              // stone length/width ratio  
      CRACK_depth = 2.,
      CRACK_zebra_scale = 1.,  // fractal shape of the fault zebra
      CRACK_zebra_amp = 0.34,
      CRACK_profile = 1.,      // fault vertical shape  1.  .2 
      CRACK_slope = 50.,       //                      10.  1.4
      CRACK_width = .001;
    

// === Voronoi =====================================================
// --- Base Voronoi. inspired by https://www.shadertoy.com/view/MslGD8

#define hash22(p)  fract( 18.5453 * sin( p * mat2(127.1,311.7,269.5,183.3)) )


// --- Voronoi distance to borders. inspired by https://www.shadertoy.com/view/ldl3W8
vec3 voronoiB( vec2 u )  // returns len + id
{
    vec2 iu = floor(u), C, P;
	float m = 1e9,d;

    for( int k=0; k < 9; k++ ) {
        vec2  p = iu + vec2(k%3-1,k/3-1),

              o = hash22(p),
      	      r = p - u + o;
		d = dot(r,r);
        if( d < m ) m = d, C = p-iu, P = r;
    }

    m = 1e9;
    
    for( int k=0; k < 25; k++ ) {
        vec2 p = iu+C + vec2(k%5-2,k/5-2),
		     o = hash22(p),
             r = p-u + o;

        if( dot(P-r,P-r)>1e-5 )
        m = min( m, .5*dot( (P+r), normalize(r-P) ) );
    }

    return vec3( m, P+u );
}

// === pseudo Perlin noise =============================================
#define rot(a) mat2(cos(a),-sin(a),sin(a),cos(a))
int MOD = 1;  // type of Perlin noise
    
// --- 2D
#define hash21(p) fract(sin(dot(p,vec2(127.1,311.7)))*43758.5453123)
float noise2(vec2 p) {
    vec2 i = floor(p);
    vec2 f = fract(p); f = f*f*(3.-2.*f); // smoothstep

    float v= mix( mix(hash21(i+vec2(0,0)),hash21(i+vec2(1,0)),f.x),
                  mix(hash21(i+vec2(0,1)),hash21(i+vec2(1,1)),f.x), f.y);
	return   MOD==0 ? v
	       : MOD==1 ? 2.*v-1.
           : MOD==2 ? abs(2.*v-1.)
                    : 1.-abs(2.*v-1.);
}


#define noise22(p) vec2(noise2(p),noise2(p+17.7))
vec2 fbm22(vec2 p) {
    vec2 v = vec2(0);
    float a = .5;
    mat2 R = rot(0.5);

    for (int i = 0; i < 6; i++, p*=2.,a/=2.) 
        p *= R,
        v += a * noise22(p);

    return v;
}

    
// ======================================================

void mainImage( out vec4 O, vec2 U )
{
    U *= 3./iResolution.y;
    U.x += iTime;                                     // for demo
    
    vec3 H0;
    O-=O;

    for(float i=0.; i<CRACK_depth ; i++) {
        vec2 V =  U / vec2(RATIO,2),                  // voronoi cell shape
             D = CRACK_zebra_amp * fbm22(U/CRACK_zebra_scale) * CRACK_zebra_scale;
        vec3  H = voronoiB( V + D ); if (i==0.) H0=H;
        float d = H.x;                                // distance to cracks
                                            // cracks
        d = min( 1., CRACK_slope * pow(max(0.,d-CRACK_width),CRACK_profile) );
  
        O += vec4(1.-d) / exp2(i);
        U *= 1.5 * rot(.37);
    }
}

*/