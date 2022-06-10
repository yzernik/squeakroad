import React, { useEffect, useState, useContext, useRef } from 'react'
import './style.scss'
import axios from 'axios'
import ContentEditable from 'react-contenteditable'
import { ICON_IMGUPLOAD } from '../../Icons'
import { Link } from 'react-router-dom'
import { API_URL } from '../../config'
import Loader from '../Loader'
import SqueakCard from '../SqueakCard'
//import MakeSqueak from '../MakeSqueak'
import MakeSqueak from '../../features/squeaks/MakeSqueak'
import Timeline from '../../features/squeaks/Timeline'



const Home = () => {

    return (
        <div className="Home-wrapper">
            <div className="Home-header-wrapper">
                <h2 className="Home-header">
                    Home
                </h2>
            </div>
            <MakeSqueak />
            <div className="Squeak-input-divider"></div>

            <Timeline />

        </div>
    )
}

export default Home
