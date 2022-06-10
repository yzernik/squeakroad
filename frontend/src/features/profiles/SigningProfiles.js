import React, { useEffect } from 'react'
import { useSelector } from 'react-redux'
import { useDispatch } from 'react-redux'
import { withRouter, Link } from 'react-router-dom'
import moment from 'moment'
import { getProfileImageSrcString } from '../../squeakimages/images';


import ProfileCard from './ProfileCard'
import Loader from '../../components/Loader'


import {
  fetchSigningProfiles,
  clearSigningProfiles,
  selectSigningProfiles,
  selectSigningProfilesStatus,
} from './profilesSlice'


const SigningProfiles = (props) => {
  const signingProfiles = useSelector(selectSigningProfiles);
  const signingProfilesStatus = useSelector(selectSigningProfilesStatus);
  const dispatch = useDispatch();

  useEffect(() => {
      window.scrollTo(0, 0)
      console.log('fetchSigningProfiles');
      dispatch(clearSigningProfiles());
      dispatch(fetchSigningProfiles(null));
  }, [])

  const renderedListItems = signingProfiles.map(profile=>{
      return <ProfileCard profile={profile}/>
      })

  return <>
            {renderedListItems}
         </>
}

export default withRouter(SigningProfiles)
