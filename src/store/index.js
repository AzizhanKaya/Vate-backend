import {configureStore} from "@reduxjs/toolkit";
import account from "../store/auth";


const store = configureStore({
	reducer: {
		account,
	}
})

export default store
