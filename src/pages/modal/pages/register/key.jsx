import { create_key, priv_to_pub } from '@/wasm/wasm';
import { useState } from 'react';
import { motion } from 'framer-motion';
import { useDispatch } from 'react-redux';
import { set_keys } from '@/store/auth';


export default function Key({setCopy}){

    const [isSpinning, setSpinning] = useState(false);
    const [isHovered, setIsHovered] = useState(false);
    const [icon, setIcon] = useState('copy');
    const [key, setKey] = useState(get_key());

    const dispatch = useDispatch();
    

    function get_key() {
        let priv_key = create_key().split(':')[1];
        const bytes = new Uint8Array(priv_key.length / 2);
        for (let i = 0; i < priv_key.length; i += 2) {
            bytes[i / 2] = parseInt(priv_key.slice(i, i + 2), 16);
        }
        const binaryString = String.fromCharCode(...bytes);
        return btoa(binaryString);
    }

    function handleKey(){

        const base64_chars="ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

        let iteration = 0;

        let final = get_key();
        console.log(final);

        if (isSpinning) return;

        setSpinning(true);
        let interval = setInterval(() => {
            setKey((prevKey) => {
                return prevKey.split("").map((letter, index) => {
                    if (index < iteration) {
                        return final[index];
                    }
                    return base64_chars[Math.floor(Math.random() * base64_chars.length)];
                }).join("");
            });
            
            
            if(iteration >= 30){ 
                clearInterval(interval);
                setSpinning(false);
                setKey(final);
                return;
            }
            
            iteration += 1;

        }, 20);
    }

    function base64ToHex(bs64) {
        const binaryString = atob(bs64);

        const byteArray = new Uint8Array(binaryString.length);
        for (let i = 0; i < binaryString.length; i++) {
            byteArray[i] = binaryString.charCodeAt(i);
        }
        let hexString = '';
        byteArray.forEach(byte => {
            hexString += byte.toString(16).padStart(2, '0');
        });
    
        return hexString;
    }

    const handleCopy = async () => {

        if (isSpinning) return;

        if (!navigator.clipboard) {
            console.error("Clipboard API not available");
            return;
        }
        
        try {
            await navigator.clipboard.writeText(key);
            dispatch(set_keys({ pub_key: priv_to_pub(base64ToHex(key)), priv_key: key }));
            setIcon('ok');
            setCopy(true);
            setTimeout(() => setIcon('copy'), 1000);
        } catch (err) {
            console.error("Failed to copy: ", err);
        }
    };

    const popOn = {
        hidden: {
            x: "120%",
            transition: {
                type: "spring",
                stiffness: 600,
                damping: 100,
            }
        },
        visible: {
            x: "0%",
            transition: {
                type: "spring",
                stiffness: 600,
                damping: 100,
            }
        }
    };

    const spinStyle = {
        transition: 'transform 0.5s ease-in-out',
        animation: isSpinning ? 'rotate 0.4s ease-in-out forwards' : 'none',
    };

    const spinAnimation = `
        @keyframes rotate {
            from {
                transform: rotate(0deg);
            }
            to {
                transform: rotate(-360deg);
            }
        }
    `;

    return(
    <div className="flex mt-10 items-center relative mx-auto">
        <div className="absolute -top-[16px] left-[30px] font-[Vate] text-[#a4a6a8] font-bold text-[13px]">Secret Key</div>
        <div className="w-[30px] h-[30px] text-xl relative">🔑</div>
        <div className="relative overflow-hidden">
            <input 
                className="w-[300px] h-9 bg-[#1b1d1f] text-white border border-[#2f3336] rounded px-2 focus:outline-none"
                style={{ fontFamily: 'Space Mono, monospace'}}
                type="text" 
                id="secretKey"
                readOnly
                value={key}
                onMouseEnter={() => setIsHovered(true)}
                onMouseLeave={() => setIsHovered(false)}
            />
            <motion.div
                variants={popOn}
                initial="hidden"
                onMouseEnter={() => setIsHovered(true)}
                onMouseLeave={() => setIsHovered(false)}
                animate={isHovered ? "visible" : "hidden"}
                exit="hidden"
                className="absolute top-1 right-1"
            >
                <button 
                    onClick={handleCopy}
                    className="rounded w-[28px] h-[28px] flex items-center justify-center border border-[#2f3336] hover:bg-[#39393bde] transition-colors bg-[#1f1f20ea]"
                >
                    {icon === 'copy' ? (
                        <svg className="mr-0.5" viewBox="0 0 24 24" transform="matrix(-1, 0, 0, 1, 0, 0)">
                            <path d="M15,17v3a1,1,0,0,1-1,1H4a1,1,0,0,1-1-1V8A1,1,0,0,1,4,7H7" fill="none" stroke="#959a9e" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.5"/>
                            <rect x="7" y="3" width="12" height="14" rx="1" fill="none" stroke="#959a9e" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.5"/>
                        </svg>
                    ) : (
                        <svg fill="#31bd4f" viewBox="0 0 24 24">
                            <path d="M12,2A10,10,0,1,0,22,12,10,10,0,0,0,12,2Zm5.676,8.237l-6,5.5a1,1,0,0,1-1.383-.03l-3-3a1,1,0,1,1,1.414-1.414l2.323,2.323,5.294-4.853a1,1,0,1,1,1.352,1.474Z"></path>
                        </svg>

                    )}
                </button>
            </motion.div>
        </div>
        <div className="pl-1 relative">
            <style>{spinAnimation}</style>
            <button className="-mr-[20px] rounded-full w-[40px] h-[40px] flex items-center justify-center hover:bg-[#eff3f427] transition-colors"
                onClick={() => handleKey()}
                style={spinStyle}
            >
                <svg className="mr-[2px]" fill="#959a9e" height="28" width="28" viewBox="0 0 75 75" transform="rotate(180)">
                    <path d="M33.51,71.01c15.49,0,28.55-10.56,32.38-24.86h9.11L61.05,22,47.11,46.15h8c-3.44,8.56-11.83,14.63-21.61,14.63-12.84,0-23.28-10.44-23.28-23.28S20.66,14.22,33.5,14.22c6.6,0,12.57,2.77,16.81,7.2l5.26-9.11c-5.9-5.18-13.62-8.32-22.07-8.32C15.03,3.99,0,19.02,0,37.5S15.03,71.01,33.51,71.01Z"/>
                </svg>
            </button>
        </div>
    </div>
)
}