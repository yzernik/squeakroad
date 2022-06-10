import React, { useEffect } from 'react'
import { useSelector } from 'react-redux'
import { useDispatch } from 'react-redux'
import { withRouter, Link } from 'react-router-dom'
import moment from 'moment'

import TwitterAccountCard from './TwitterAccountCard'
import Loader from '../../components/Loader'


import {
  fetchTwitterAccounts,
  clearTwitterAccounts,
  selectTwitterAccounts,
  selectTwitterAccountsStatus,
} from './twitterAccountsSlice'


const TwitterAccounts = (props) => {
  const twitterAccounts = useSelector(selectTwitterAccounts);
  const twitterAccountsStatus = useSelector(selectTwitterAccountsStatus);
  const dispatch = useDispatch();

  useEffect(() => {
      window.scrollTo(0, 0)
      console.log('fetchTwitterAccounts');
      dispatch(clearTwitterAccounts());
      dispatch(fetchTwitterAccounts(null));
  }, [])

  const renderedListItems = twitterAccounts.map(twitterAccount=>{
      return <TwitterAccountCard twitterAccount={twitterAccount}/>
      })

  return <>
            {renderedListItems}
         </>
}

export default withRouter(TwitterAccounts)
