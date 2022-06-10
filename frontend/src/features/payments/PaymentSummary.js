import React , { useEffect, useContext, useState } from 'react'
import {  withRouter, Link } from 'react-router-dom'
import { ICON_SEARCH } from '../../Icons'
import Loader from '../../components/Loader'

import { useSelector } from 'react-redux'
import { useDispatch } from 'react-redux'


import {
  fetchPaymentSummary,
  selectPaymentSummary,
} from './paymentsSlice'


const PaymentSummary = (props) => {
  const paymentSummary = useSelector(selectPaymentSummary)
  const dispatch = useDispatch()

  useEffect(() => {
      window.scrollTo(0, 0)
      console.log('fetchPaymentSummary');
      dispatch(fetchPaymentSummary());
  }, [])


  return (
      <>
          {paymentSummary ?
            <div className="feed-trending-card">
                <h3 className="feed-card-header">Payments</h3>
                <div onClick={()=>props.history.push('/app/payments')}className="feed-card-trend">
                    <div>Amount Spent</div>
                    <div>{paymentSummary.getAmountSpentMsat() / 1000} sats</div>
                    <div>{paymentSummary.getNumSentPayments()} squeaks</div>
                </div>
                <div onClick={()=>props.history.push('/app/payments')}className="feed-card-trend">
                    <div>Amount Earned</div>
                    <div>{paymentSummary.getAmountEarnedMsat() / 1000} sats</div>
                    <div>{paymentSummary.getNumReceivedPayments()} squeaks</div>
                </div>
            </div> :
            <Loader/>
          }
      </>
      )
}

export default withRouter(PaymentSummary)
