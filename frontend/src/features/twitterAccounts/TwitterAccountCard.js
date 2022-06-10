import React, { useEffect, useState } from 'react'
import { useSelector } from 'react-redux'
import { useDispatch } from 'react-redux'
import { withRouter, Link } from 'react-router-dom'
import moment from 'moment'
import { ICON_SETTINGS } from '../../Icons'
import { getProfileImageSrcString } from '../../squeakimages/images';


import SqueakCard from '../../components/SqueakCard'
import Loader from '../../components/Loader'

import {
  setDeleteTwitterAccount,
} from './twitterAccountsSlice'


const TwitterAccountCard = (props) => {
  const [moreMenu, setMoreMenu] = useState(false)
  const [styleBody, setStyleBody] = useState(false)
  const [saved, setSaved] = useState(false)
  const [deleteModalOpen, setDeleteModalOpen] = useState(false)

  const twitterAccount = props.twitterAccount;
  const profile = twitterAccount.getProfile();
  const moreMenuId = `twitterAccountMoreMenu_${twitterAccount.getTwitterAccountId()}`;
  const dispatch = useDispatch();

  // const followUser = (e, id) => {
  //     e.stopPropagation()
  //     console.log('Follow clicked');
  //     dispatch(setFollowProfile(id));
  // }
  //
  // const unfollowUser = (e,id) => {
  //     e.stopPropagation()
  //     console.log('Unfollow clicked');
  //     dispatch(setUnfollowProfile(id));
  // }

  const deleteTwitterAccount = (e) => {
    e.stopPropagation()
    console.log('Delete clicked');
    dispatch(setDeleteTwitterAccount({
      twitterAccountId: twitterAccount.getTwitterAccountId(),
    }));
  }

  const openMore = () => { setMoreMenu(!moreMenu) }

  const handleMenuClick = (e) => { e.stopPropagation() }

  // const toggleDeleteModal = () => {
  //   setStyleBody(!styleBody)
  //   setSaved(false)
  //   setTimeout(()=>{ setDeleteModalOpen(!deleteModalOpen) },20)
  // }

  console.log(twitterAccount);
  console.log(twitterAccount.getIsForwarding());

  return <div onClick={(e)=>e.stopPropagation()} key={profile.getPubkey()} className="search-result-wapper">
    <Link to={`/app/profile/${profile.getPubkey()}`} className="search-userPic-wrapper">
      <div className="search-userPic">
        <img style={{borderRadius:'50%', minWidth:'49px'}} width="100%" height="49px" src={`${getProfileImageSrcString(profile)}`}/>
      </div>
    </Link>
    <div className="search-user-details">
      <div className="search-user-warp">
        <div className="search-user-info">
          <div className="search-user-name">{profile.getProfileName()}</div>
          <div className="search-user-username">@{profile.getPubkey()}</div>
        </div>
      </div>
      <div className="search-user-username"><b>Twitter Handle</b>: <a href={`https://twitter.com/${twitterAccount.getHandle()}`} target="_blank" rel="noopener noreferrer" style={{color: "blue", fontWeight: 'bold'}}>
        {twitterAccount.getHandle()}
      </a></div>
      <div className="search-user-bio">
        &nbsp;
      </div>
      {
        twitterAccount.getIsForwarding() ?
        <div style={{color: "green", fontWeight: 'bold'}}>Forwarding Successfully</div>  :
          <div style={{color: "red", fontWeight: 'bold'}}>Forwarding Failing</div>
        }
      </div>
      <div id={moreMenuId} onClick={openMore} className="Nav-link">
        <div className={"Nav-item-hover"}>
          <ICON_SETTINGS  />
        </div>
        <div onClick={()=>openMore()} style={{display: moreMenu ? 'block' : 'none'}} className="more-menu-background">
          <div className="more-modal-wrapper">
            {moreMenu ?
              <div style={{
                  top: document.getElementById(moreMenuId) && `${document.getElementById(moreMenuId).getBoundingClientRect().top - 40}px`,
                  left: document.getElementById(moreMenuId) && `${document.getElementById(moreMenuId).getBoundingClientRect().left}px`,
                  height: '40px',
                }} onClick={(e)=>handleMenuClick(e)} className="more-menu-content">
                <div onClick={deleteTwitterAccount} className="more-menu-item">
                  <span>Delete</span>
                </div>
              </div> : null }
            </div>
          </div>
        </div>
      </div>
    }

    export default withRouter(TwitterAccountCard)
