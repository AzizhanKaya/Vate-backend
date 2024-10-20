import { createBrowserRouter } from "react-router-dom";
import Home from "../pages/home";
import Explore from "../pages/explore";
import MainLayout from "../layouts/main";
import Profile from "../pages/profile";
import OnTheWay from "../pages/on-the-way"

const routes = createBrowserRouter([

    {

        path: '/',
        element: <MainLayout />,
        children: [

            {
                index: true,
                element: <Home />
            },

            {
                path: 'explore',
                element: <OnTheWay />
            },
            {
                path: 'notifications',
                element: <OnTheWay />
            },
            {
                path: ':username',
                element: <OnTheWay />
            }
        ]

    }
    
])

export default routes