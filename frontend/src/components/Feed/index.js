import React , { useEffect, useContext, useState } from 'react'
import './style.scss'
import {  withRouter, Link } from 'react-router-dom'
import { ICON_SEARCH } from '../../Icons'
import Loader from '../Loader'
import PaymentSummary from '../../features/payments/PaymentSummary'


const Feed = (props) => {

const [searchText, setSearchText] = useState('');

const changeSearchText = (param) => {
    setSearchText(param);
}

const searchOnEnter = (e) => {
    if (e.keyCode === 13) {
      if(searchText.length>0){
        console.log("Goto")
        console.log(searchText)
        goToNewSearch(searchText);
        setSearchText('');
      }
    }
}

const goToNewSearch = (newSearchText) => {
    props.history.push(`/app/search?q=${newSearchText}`);
}

return(
    <div className="feed-wrapper">
      <div className="explore-search-wrapper">
        <div className="explore-search-icon">
            <ICON_SEARCH/>
        </div>
        <div className="explore-search-input">
          <input value={searchText} onKeyDown={(e)=>searchOnEnter(e)} onChange={(e)=>changeSearchText(e.target.value)}  placeholder="Search Squeaks" type="text" name="search"/>
        </div>
      </div>

      <PaymentSummary />
    </div>
    )
}

export default withRouter(Feed)
