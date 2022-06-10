import React, { useEffect } from 'react'
import { useSelector } from 'react-redux'
import { useDispatch } from 'react-redux'
import { withRouter, Link } from 'react-router-dom'
import moment from 'moment'
import { getProfileImageSrcString } from '../../squeakimages/images';


import ProfileCard from './ProfileCard'
import Loader from '../../components/Loader'


import {
  fetchContactProfiles,
  clearContactProfiles,
  selectContactProfiles,
  selectContactProfilesStatus,
} from './profilesSlice'


const ContactProfiles = (props) => {
  const contactProfiles = useSelector(selectContactProfiles);
  const contactProfilesStatus = useSelector(selectContactProfilesStatus);
  const dispatch = useDispatch();

  useEffect(() => {
      window.scrollTo(0, 0)
      console.log('fetchContactProfiles');
      dispatch(clearContactProfiles());
      dispatch(fetchContactProfiles());
  }, [])

  const renderedListItems = contactProfiles.map(profile=>{
      return <ProfileCard profile={profile}/>
      })

  return <>
            {renderedListItems}
         </>
}

export default withRouter(ContactProfiles)
