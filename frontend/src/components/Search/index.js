import React, { useEffect, useState, useContext, useRef, useMemo } from 'react'
import { withRouter, useLocation } from 'react-router-dom';
import './style.scss'
import axios from 'axios'
import ContentEditable from 'react-contenteditable'
import { ICON_IMGUPLOAD, ICON_SEARCH } from '../../Icons'
import { Link } from 'react-router-dom'
import { API_URL } from '../../config'
import Loader from '../Loader'
import SqueakCard from '../SqueakCard'
import SearchResults from '../../features/squeaks/SearchResults'



const Search = (props) => {

    return (
        <div className="Home-wrapper">
          <SearchResults />
        </div>
    )
}

export default withRouter(Search)
