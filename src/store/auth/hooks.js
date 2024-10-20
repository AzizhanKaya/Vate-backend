import {useSelector, useDispatch} from "react-redux";
import { setAccount, setProfilePic, set_keys } from '@/store/auth';
export const useAccount = () => useSelector(state => state.account.Account);

export const useLogout = () => {
    const dispatch = useDispatch();

    dispatch(setAccount({ username: null }));
    dispatch(setProfilePic({ profile_pic: null }));
    dispatch(set_keys({ priv_key: null, pub_key: null }));

};