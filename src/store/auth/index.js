import { createSlice } from "@reduxjs/toolkit";

const initialState = {
    Account: {
        username: null,
        profile_pic: null,
        pub_key: null,
        priv_key: null
    }
};

const accountSlice = createSlice({
    name: 'account',
    initialState,
    reducers: {
        setAccount: (state, action) => {
            state.Account.username = action.payload.username;
        },
        setProfilePic: (state, action) => {
            state.Account.profile_pic = action.payload.profile_pic;
        },
        set_keys: (state, action) => {
            state.Account.priv_key = action.payload.priv_key;
            state.Account.pub_key = action.payload.pub_key; 
        }
    }
});

export const { setAccount, setProfilePic, set_keys} = accountSlice.actions;

export default accountSlice.reducer;
