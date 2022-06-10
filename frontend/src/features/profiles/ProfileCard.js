import React, { useEffect } from 'react'
import { useSelector } from 'react-redux'
import { useDispatch } from 'react-redux'
import { withRouter, Link } from 'react-router-dom'
import moment from 'moment'
import { getProfileImageSrcString } from '../../squeakimages/images';


import SqueakCard from '../../components/SqueakCard'
import Loader from '../../components/Loader'


import {
  setFollowProfile,
  setUnfollowProfile,
} from './profilesSlice'


const ProfileCard = (props) => {
  const profile = props.profile;
  const dispatch = useDispatch();

  const followUser = (e, id) => {
      e.stopPropagation()
      console.log('Follow clicked');
      dispatch(setFollowProfile(id));
  }

  const unfollowUser = (e,id) => {
      e.stopPropagation()
      console.log('Unfollow clicked');
      dispatch(setUnfollowProfile(id));
  }

      return <Link onClick={(e)=>e.stopPropagation()} to={`/app/profile/${profile.getPubkey()}`} key={profile.getPubkey()} className="search-result-wapper">
          <div className="search-userPic-wrapper">
            <div className="search-userPic">
              <img style={{borderRadius:'50%', minWidth:'49px'}} width="100%" height="49px" src={`${getProfileImageSrcString(profile)}`}/>
          </div>
          </div>
          <div className="search-user-details">
              <div className="search-user-warp">
                  <div className="search-user-info">
                      <div className="search-user-name">{profile.getProfileName()}</div>
                      <div className="search-user-username">@{profile.getPubkey()}</div>
                  </div>
                  <div onClick={(e)=>{
                      e.preventDefault();
                      profile.getFollowing() ?
                      unfollowUser(e,profile.getProfileId()) :
                      followUser(e,profile.getProfileId())
                  }} className={profile.getFollowing() ? "follow-btn-wrap unfollow-switch":"follow-btn-wrap"}>
                  <span><span>{profile.getFollowing() ? 'Following' : 'Follow'}</span></span>
              </div>
          </div>
          <div className="search-user-bio">
                &nbsp;
          </div>
        </div>
      </Link>
}

export default withRouter(ProfileCard)
