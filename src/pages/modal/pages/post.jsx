import { useAccount } from '@/store/auth/hooks';
import { get_time, sign, get_hash } from '@/wasm/wasm'
import { useEffect, useState } from 'react';
import styles from './assets/Post.module.css';
export default function Post({setExitModal}){

    const Account = useAccount();
    const [timestamp, setTimestamp] = useState(get_time().toString());
    const [postButton, setPostButton] = useState(false);
    const [textBox, setTextBox] = useState('');
    const [textBoxType, setTextBoxType] = useState(false);
    const [isTextBox, setIsTextBox] = useState(false);
    const [post, setPost] = useState(false);

    function hexToBase64(hex) {
        if (hex === null) return;
        
        const bytes = [];
        for (let i = 0; i < hex.length; i += 2) {
            bytes.push(parseInt(hex.substr(i, 2), 16));
        }
        const binaryString = String.fromCharCode(...bytes);
        return btoa(binaryString);
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

    function handleCopy(copy){
        navigator.clipboard.writeText(copy)
        .catch(err => {
            console.error('Failed to copy text: ', copy);
        });
    }
    
    function timestampToDate(timestamp) {

        const date = new Date(timestamp * 1000);
        
        const hours = String(date.getHours()).padStart(2, '0');
        const minutes = String(date.getMinutes()).padStart(2, '0');
        const seconds = String(date.getSeconds()).padStart(2, '0');
        const month = String(date.getMonth() + 1).padStart(2, '0');
        const day = String(date.getDate()).padStart(2, '0');
        const year = date.getFullYear();
        
        
        return `${hours}:${minutes}:${seconds} - ${month}/${day}/${year}`;
    }

    useEffect(() => {
        const intervalId = setInterval(() => {
            setTimestamp(get_time().toString()); 
        }, 1000);

        
        return () => clearInterval(intervalId);
    }, []);

    function handlePost(post){
        setPost(post);
        if (post != ''){
            setPostButton(true);
        }else{
            setPostButton(false);
        }
    }
    async function handlePostButton(){
        
        if (post == '') return;

        const time = timestamp;
        
        try{
            const payload = `${Account.pub_key}:Welcome:${post}:${time}`;
            const past_hash = get_hash(payload);
            const hash = get_hash(`${past_hash}:${Account.pub_key}:Welcome:${post}:${time}`);
            
            const signed = sign(base64ToHex(Account.priv_key), hash);

            const data = {
                past_hash: past_hash,
                pub_key: Account.pub_key,
                subject: 'Welcome',
                message: post,
                time: time,
                sign: signed,
            };

            const response = await fetch('http://192.168.1.25:3000/post', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify(data)
            });

            if (response.ok) {
                setTextBox('Posted successfully');
                setTextBoxType(true);
                setIsTextBox(true);
                return true;
            }else{
                const errorText = await response.text();
                setTextBox(errorText);
                setTextBoxType(false);
                setIsTextBox(true);
                console.log("Server Error while registering: ", errorText);
                return false;
            }
        }
        catch(error){
            setTextBox(error.message);
            setTextBoxType(false);
            setIsTextBox(true);
            console.log("Register Error: ",error);
            return false;
        }
    }

    function delay(ms) {
        return new Promise(resolve => setTimeout(resolve, ms));
    }

    async function handlePostButtonClick(){
        if(await handlePostButton()){
            await delay(1000);
            setExitModal(true);
        }else{
            await delay(2000);
            setIsTextBox(false);
        }
    }
    
    return(
    <div className="flex flex-col items-center p-8 mt-8">
        <div className="font-[Vate] text-[22px] text-center w-full p-2">
            Son bir adım!
        </div>
        <div className="w-[500px] border-t-2 border-[#2f3336]">
            <div className="px-4 pt-3 gap-3 border border-[#2f3336] flex rounded-xl my-3 bg-[#0a0a0a] w-full">
                <img src={Account.profile_pic} className="w-10 h-10 rounded-full object-cover" alt="" />
                <div>
                    <header className="leading-5 flex gap-2 items-center mb-0.5">
                        <a className="hover:underline font-bold cursor-pointer">
                            {Account.username}
                        </a>
                        <div className="text-[#585858] flex items-center gap-1.5">
                            <div className="overflow-hidden whitespace-nowrap text-ellipsis w-[150px] cursor-pointer"
                            onClick={() => handleCopy(hexToBase64(Account.pub_key))}
                            >
                                @{hexToBase64(Account.pub_key)}
                            </div>
                            <div>‧</div>
                            <div>{timestampToDate(timestamp)}</div>
                        </div>
                        
                        
                        
                    </header>

                    <div className="py-3 w-[400px] h-[200px] mb-3">
            
                        <textarea
                            className="w-full h-full bg-[#0a0a0a] text-white border text-[16px] border-[#2f3336] rounded p-2 focus:outline-none resize-none srollbar-hidden"
                            style={{ scrollbarWidth: 'none' }}
                            placeholder="What's going on..."
                            maxLength={130}
                            onChange={(event) => handlePost(event.target.value)}
                        />
                        

                    </div>
                </div>
            </div>
        </div>
        <div className="flex justify-center">
                <div className={`${styles['text-box']} ${isTextBox ? 'fade-in' : 'fade-out'} ${textBoxType ? styles['success-box'] : styles['warn-box']}`}>
                <svg viewBox="-20 -20 550 550" width="28" height="28" fill="#fff">
                    <circle cx="256" cy="256" r="246" fill="none" stroke="#fff" strokeLinecap="round" strokeLinejoin="round" strokeWidth="40"/>
                    <line x1="371.47" y1="140.53" x2="140.53" y2="371.47" stroke="#fff" strokeLinecap="round" strokeLinejoin="round" strokeWidth="40"/>
                    <line x1="371.47" y1="371.47" x2="140.53" y2="140.53" stroke="#fff" strokeLinecap="round" strokeLinejoin="round" strokeWidth="40"/>
                </svg>
                <div className="border-r-2 mx-2 h-[25px]"></div>
                <div className="font-semibold">
                    {textBox}
                </div>
            </div>
        </div>

        
        
        <button
        className={`${styles['post-button']} ${postButton ? 'fade-in' : 'fade-out'} `}
        onClick={handlePostButtonClick}
        >
          Post
        </button>
        
        
    </div>
    )
}