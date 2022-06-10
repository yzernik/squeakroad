import React, { useEffect, useState, useContext, useRef, useMemo } from 'react'
import { withRouter, useLocation } from 'react-router-dom';
import ContentEditable from 'react-contenteditable'
import { ICON_IMGUPLOAD, ICON_SEARCH } from '../../Icons'
import { Link } from 'react-router-dom'
import { API_URL } from '../../config'

import { useSelector } from 'react-redux'
import { useDispatch } from 'react-redux'

import SqueakCard from '../../components/SqueakCard'
import Loader from '../../components/Loader'


import {
  clearSearch,
  fetchSearch,
  selectSearchSqueaks,
  selectLastSearchSqueak,
  selectSearchSqueaksStatus,
} from '../squeaks/squeaksSlice'

import store from '../../store'


const SearchResults = (props) => {
  const squeaks = useSelector(selectSearchSqueaks);
  const loadingStatus = useSelector(selectSearchSqueaksStatus)
  const lastSqueak = useSelector(selectLastSearchSqueak)
  const dispatch = useDispatch()

  const { search } = useLocation();
  const [searchText, setSearchText] = useState('');

  const q = useMemo(() => {
    const p = new URLSearchParams(search);
    const q = p.get('q');
    return q ? decodeURIComponent(q) : '';
  }, [search]);

  const scrollSize = 10;

  useEffect(() => {
    window.scrollTo(0, 0)
    if (q && q.length > 0) {
      setSearchText(q);
      console.log('fetchTodos');
      dispatch(clearSearch());
      const values = {
        searchText: q,
        limit: scrollSize,
        lastSqueak: null,
      }
      dispatch(fetchSearch(values));
    }
  }, [q])

  const searchOnEnter = (e) => {
    if (e.keyCode === 13) {
      if(searchText.length>0){
        goToNewSearch(searchText);
      }
    }
  }

  const goToNewSearch = (newSearchText) => {
    props.history.push(`/app/search?q=${newSearchText}`);
  }


  const changeSearchText = (param) => {
    setSearchText(param);
  }

  const getLastSqueak = (squeakLst) => {
    if (squeakLst == null) {
      return null;
    } if (squeakLst.length === 0) {
      return null;
    }
    return squeakLst.slice(-1)[0];
  };

  const getMoreSqueaks = () => {
    let lastSqueak = getLastSqueak(squeaks);
    const values = {
      searchText: searchText,
      limit: scrollSize,
      lastSqueak: lastSqueak,
    }
    dispatch(fetchSearch(values));
  }


  const renderedListItems = squeaks.map((squeak) => {
    return <SqueakCard squeak={squeak} key={squeak.getSqueakHash()} id={squeak.getSqueakHash()} user={squeak.getAuthor()} />
  })

  return <>
  <div className="explore-header">
    <div className="explore-search-wrapper">
      <div className="explore-search-icon">
        <ICON_SEARCH/>
      </div>
      <div className="explore-search-input">
        <input value={searchText} onKeyDown={(e)=>searchOnEnter(e)} onChange={(e)=>changeSearchText(e.target.value)} placeholder="Search Squeaks" type="text" name="search"/>
      </div>
    </div>
  </div>
  <div className="Squeak-input-divider"></div>
  {squeaks.map(t => {
    return <SqueakCard squeak={t} key={t.getSqueakHash()} id={t.getSqueakHash()} user={t.getAuthor()} />
  })}

  {loadingStatus === 'loading' ?
    <div className="todo-list">
      <Loader />
    </div>
    :
    <div onClick={() => getMoreSqueaks()} className='squeak-btn-side squeak-btn-active'>
      LOAD MORE
    </div>
  }
  </>
}

export default withRouter(SearchResults)
