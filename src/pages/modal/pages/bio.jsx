import { useState } from 'react';
import styles from './assets/Bio.module.css';
import { useAccount } from '@/store/auth/hooks';
import { sign , get_hash } from '@/wasm/wasm'
import { useDispatch } from 'react-redux';
import { setAccount, setProfilePic } from '@/store/auth';

export default function Bio({goToPage}) {
    const [bio, setBio] = useState('');
    const [username, setUsername] = useState('');
    const [warning, setWarning] = useState('');
    const [is_warning, setIsWarning] = useState(false);
    const [nextButton, setNextButton] = useState(false);
    const [profileImage, setProfileImage] = useState(null);
    const [imagePreview, setImagePreview] = useState(null);
    const [isImage, setisImage] = useState(false);
    const [checkBioValid, setCheckBioValid] = useState(false);
    const [checkUserValid, setCheckUserValid] = useState(false);

    const Account = useAccount();
    const dispatch = useDispatch();

    const handleBioChange = (event) => {
        const { value } = event.target;
        const lines = value.split('\n');

        if (lines.length <= 4) {
            setBio(value);
        } else {
            const newBio = lines.slice(0, 4).join('\n');
            setBio(newBio);
        }
    };

    const handleUsernameChange = (event) => {
        setUsername(event.target.value);
    };

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

    async function sha256(bytes) {
        const buffer = new Uint8Array(bytes).buffer;
        const hashBuffer = await crypto.subtle.digest('SHA-256', buffer);
        const hashArray = new Uint8Array(hashBuffer);
        const hashHex = Array.from(hashArray)
            .map(byte => byte.toString(16).padStart(2, '0'))
            .join('');
    
        return hashHex;
    }

    function checkUserName() {

        if (username == '') {setWarning('Username can not be empty'); return false;}

        if (username.length < 3) {setWarning('Username lenght is too short'); return false;}

        if (username.includes(' ')) {setWarning('Username can not contain whitespaces'); return false;}

        return true;
    }
    
    function checkBio() {

        if (bio == '') {setWarning('Bio can not be empty'); return false;}
        
        return true;
    }

    const checkUser = (field) => {
        let userValid = checkUserValid;
        let bioValid = checkBioValid;

        if (field === 'Username') {
            userValid = checkUserName();
            setIsWarning(!userValid);
            setCheckUserValid(userValid);
        } else if (field === 'Bio') {
            bioValid = checkBio();
            setIsWarning(!bioValid);
            setCheckBioValid(bioValid);
        }
        setNextButton(userValid && bioValid);
    };

    const handleNext = async () => {
        if (!(checkUserName() && checkBio())) return;

        try {
            const payload = `${Account.pub_key}:${username}:${bio}`;
            const hash = get_hash(payload);
            
            const signed = sign(base64ToHex(Account.priv_key), hash);

            const response = await fetch('http://192.168.1.25:3000/register', {
                method: 'POST',
                body: `${payload}:${signed}`,
            });

            if (response.ok) {

                dispatch(setAccount({ username: username }));

                if (profileImage) {

                    const upload_response = await uploadProfilePic();

                    if (upload_response.ok){
                        setIsWarning(false);
                        const img_type = profileImage.type.split('/')[1];
                        dispatch(setProfilePic({ profile_pic: `http://192.168.1.25:3000/${Account.pub_key}/pp.${img_type}`}));
                        goToPage(3);
                    } else {
                        
                        setWarning(await upload_response.text());
                        setIsWarning(true);
                    }
                    
                }
                else {
                    dispatch(setProfilePic({ profile_pic: '/profile-image.png'}));
                    setIsWarning(false);
                    goToPage(3);
                    return;
                }
                
                return;
            }
            else
            {
                setWarning(await response.text());
                setIsWarning(true);
                return;
            }
            
        } catch (error) {
            console.error('Register Error:', error);
        }
    };

    async function uploadProfilePic() {

        try {
            
            const img_type = profileImage.type.split('/')[1];
            
            const img_bytes = await (async () => {
                const arrayBuffer = await profileImage.arrayBuffer();
                return arrayBuffer;
            })();

            const hash = await sha256(img_bytes);
            const signed = await sign(base64ToHex(Account.priv_key), hash);
            console.log(img_bytes);
            const url = `http://192.168.1.25:3000/upload/${Account.pub_key}/pp.${img_type}`;
            const response = await fetch(url, {
                method: 'POST',
                headers: {
                    'Sign': signed,
                    'Content-Type': 'application/octet-stream',
                },
                body: img_bytes
            });
            
            return response;
        } catch (error) {
            console.error('Uploading Error:', error);
        }
    }

    const addProfilePic = (event) => {
        const file = event.target.files[0];
        if (file) {
            const reader = new FileReader();
            reader.onloadend = () => {
                setImagePreview(reader.result);
                setProfileImage(file);
            };
            reader.readAsDataURL(file);
            setisImage(true);
        }
    };


    return (
        <div className="w-full h-full" style={{ willChange: 'transform'}} >
            <div className="flex flex-col pt-[100px]">
                <div className="font-[Vate] p-2 text-[22px] text-center w-full">
                    Biraz kendinden bahset!
                </div>
                <div className="px-[45px]">
                    <div className="flex border-t-2 border-[#2f3336] justify-center">
                        <div>
                            <div className={`${styles['profile-img']} mt-5 ${isImage ? '' : 'none'}`}
                            style={{
                                backgroundImage: isImage ? '' : 'url("/profile-image.png")',
                            }}>
                                {imagePreview && (
                                    <img
                                        src={imagePreview}
                                        style={{
                                            width: '100%',
                                            height: '100%',
                                            objectFit: 'cover',
                                            borderRadius: '50%',
                                        }}
                                    />
                                )}
                                <input
                                    type="file"
                                    accept=".png,.jpg"
                                    onChange={addProfilePic}
                                />
                            </div>
                        </div>
                        <div className="flex flex-col gap-2 mt-5">
                            <input
                                className="w-[320px] h-8 bg-[#1b1d1f] text-white border text-[18px] border-[#2f3336] rounded px-2 focus:outline-none font-semibold" 
                                type="text" 
                                style={{ scrollbarWidth: 'none' }}
                                placeholder="Username"
                                value={username}
                                onChange={handleUsernameChange}
                                maxLength={20}
                                onBlur={() => checkUser("Username")}
                            />
                            <textarea
                                className="w-[320px] h-[100px] bg-[#1b1d1f] text-white border text-[16px] border-[#2f3336] rounded p-2 focus:outline-none resize-none srollbar-hidden"
                                style={{ scrollbarWidth: 'none' }}
                                placeholder="Your Biography"
                                value={bio}
                                onChange={handleBioChange}
                                rows={4}
                                maxLength={130}
                                onBlur={() => checkUser("Bio")}
                            />
                        </div>
                    </div>
                </div>
                <div className="flex justify-center">
                    <div className={`${styles['warn-box']} ${is_warning ? 'fade-in' : 'fade-out'}`}
                    >
                        <svg viewBox="-20 -20 550 550" width="28" height="28" fill="#fff">
                            <circle cx="256" cy="256" r="246" fill="none" stroke="#fff" strokeLinecap="round" strokeLinejoin="round" strokeWidth="40"/>
                            <line x1="371.47" y1="140.53" x2="140.53" y2="371.47" stroke="#fff" strokeLinecap="round" strokeLinejoin="round" strokeWidth="40"/>
                            <line x1="371.47" y1="371.47" x2="140.53" y2="140.53" stroke="#fff" strokeLinecap="round" strokeLinejoin="round" strokeWidth="40"/>
                        </svg>
                        <div className="border-r-2 mx-2 h-[25px]"></div>
                        <div className="font-semibold">
                            {warning}
                        </div>
                    </div>
                </div>
                
            </div>

        
    <button
        className={`next-button ${nextButton ? 'fade-in' : 'fade-out'}`}
        onClick={handleNext}
    >
          Next
    </button>
        </div>
    );
}
