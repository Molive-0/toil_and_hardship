#version 450

layout(location=0)out vec4 fragColor;

vec3 logo(vec2 u,float time){
    mat2 x=mat2(cos(time),sin(time),-sin(time),cos(time));
    mat2 y=mat2(sin(time),cos(time),-cos(time),sin(time));
    float l=length(u);
    
    if(l<.17){
        return vec3(1.);
    }else if(l<.22){
        u=x*u;
        if(abs(u.y)<.02){
            return vec3(1.);
            
        }else{
            return vec3(.82,.0,.1);
        }
    }else if(l<.25){
        return vec3(1.);
    }else if(l<.4){
        u=y*u;
        if(l>.32){
            float a=fract((atan(u.y,u.x)/(atan(-1.)*4.)+.5)*3.);
            if((l>.37)&&(a>.85-(l*2.)&&a<.15+(l*2.))){
                return vec3(1.);
            }else if(a>.275-l/2.&&a<.725+l/2.){
                return vec3(1.);
            }
        }
        if(abs(u.y)<.02){
            return vec3(1.);
        }else if(u.y<0.){
            return vec3(.35,.32,.32);
        }else{
            return vec3(.12);
        }
    }else if(l<.44){
        return vec3(1.);
    }else if(l<.5){
        return vec3(.12);
    }
    
    //flashy colours
    return vec3((sin(u.y+time*2.)+1.)/2.,(sin(u.x+time*3.)+1.)/2.,(sin(u.y+time*5.)+1.)/2.);
}

void main(){
    vec2 iResolution=vec2(1920.,1080.);
    float iTime=0;
    // Normalized pixel coordinates (from -0.5 to 0.5 vertically, -AR to AR horizontally)
    vec2 uv=gl_FragCoord.xy/iResolution.y;
    uv.x-=iResolution.x/iResolution.y/2.;
    uv.y-=.5;
    
    // Antialias
    float low=1./iResolution.y/2.;;
    vec3 o=vec3(low,-low,0.);
    vec3 col=(logo(uv+o.xx,iTime)
    +logo(uv+o.xy,iTime)
    +logo(uv+o.yx,iTime)
    +logo(uv+o.yy,iTime)
    +logo(uv+o.yz,iTime)
    +logo(uv+o.zy,iTime)
    +logo(uv+o.zz,iTime)
    +logo(uv+o.zx,iTime)
    +logo(uv+o.xz,iTime))/9.;
    
    // Output to screen
    fragColor=vec4(col,1.);
}